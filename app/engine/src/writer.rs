use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use itertools::Itertools;
use typeshare_model::prelude::*;

use crate::{args::OutputLocation, parser::ParsedData, topsort::topsort};

pub fn write_output<'c>(
    lang: &impl Language<'c>,
    crate_parsed_data: HashMap<Option<CrateName>, ParsedData>,
    dest: &OutputLocation<'_>,
) -> anyhow::Result<()> {
    match dest {
        OutputLocation::File(file) => {
            // merge all data together
            let mut parsed_data = crate_parsed_data
                .into_values()
                .reduce(|mut data, new_data| {
                    data.merge(new_data);
                    data
                })
                .context("called `write_output` with no data")?;

            parsed_data.sort_contents();
            write_single_file(lang, file, &parsed_data)
        }
        OutputLocation::Folder(directory) => {
            // TODO: compute import candidates here

            let crate_parsed_data = crate_parsed_data
                .into_iter()
                .map(|(crate_name, mut data)| match crate_name {
                    Some(crate_name) => {
                        data.sort_contents();
                        Ok((crate_name, data))
                    }
                    None => anyhow::bail!(
                        "got files with unknown crates; all files \
                         must be in crates in multi-file mode"
                    ),
                })
                .try_collect()?;

            write_multiple_files(lang, directory, &crate_parsed_data)
        }
    }
}

/// Write multiple module files.
pub fn write_multiple_files<'c>(
    lang: &impl Language<'c>,
    output_folder: &Path,
    crate_parsed_data: &HashMap<CrateName, ParsedData>,
) -> anyhow::Result<()> {
    let mut output_files = Vec::with_capacity(crate_parsed_data.len());

    // TODO: multithread this part
    for (crate_name, parsed_data) in crate_parsed_data {
        let file_path = output_folder.join(&lang.output_filename_for_crate(&crate_name));

        let mut output = Vec::new();

        generate_types(
            lang,
            &mut output,
            parsed_data,
            FilesMode::Multi(&crate_name),
        )
        .with_context(|| format!("error generating typeshare types for crate {crate_name}"))?;

        check_write_file(&file_path, output).with_context(|| {
            format!(
                "error writing generated typeshare types for crate {crate_name} to '{}'",
                file_path.display()
            )
        })?;

        output_files.push((crate_name, file_path));
    }

    output_files.sort_by_key(|&(crate_name, _)| crate_name);

    lang.write_additional_files(
        output_folder,
        output_files
            .iter()
            .map(|(crate_name, file_path)| (*crate_name, file_path.as_path())),
    )
    .context("failed to write additional files")?;

    Ok(())
}

/// Write all types to a single file.
pub fn write_single_file<'c>(
    lang: &impl Language<'c>,
    file_name: &Path,
    parsed_data: &ParsedData,
) -> Result<(), anyhow::Error> {
    let mut output = Vec::new();

    generate_types(lang, &mut output, parsed_data, FilesMode::Single)
        .context("error generating typeshare types")?;

    let outfile = Path::new(file_name).to_path_buf();
    check_write_file(&outfile, output)
        .context("error writing generated typeshare types to file")?;
    Ok(())
}

/// Write the file if the contents have changed.
fn check_write_file(outfile: &PathBuf, output: Vec<u8>) -> anyhow::Result<()> {
    match fs::read(outfile) {
        Ok(buf) if buf == output => {
            // avoid writing the file to leave the mtime intact
            // for tools which might use it to know when to
            // rebuild.
            eprintln!("Skipping writing to {outfile:?} no changes");
            return Ok(());
        }
        _ => {}
    }

    if !output.is_empty() {
        let out_dir = outfile
            .parent()
            .context(format!("Could not get parent for {outfile:?}"))?;
        // If the output directory doesn't already exist, create it.
        if !out_dir.exists() {
            fs::create_dir_all(out_dir).context("failed to create output directory")?;
        }

        fs::write(outfile, output).context("failed to write output")?;
    }
    Ok(())
}

/// An enum that encapsulates units of code generation for Typeshare.
/// Analogous to `syn::Item`, even though our variants are more limited.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorrowedRustItem<'a> {
    /// A `struct` definition
    Struct(&'a RustStruct),
    /// An `enum` definition
    Enum(&'a RustEnum),
    /// A `type` definition or newtype struct.
    Alias(&'a RustTypeAlias),
    /// A `const` definition
    Const(&'a RustConst),
}

impl BorrowedRustItem<'_> {
    pub fn name(&self) -> &str {
        match *self {
            BorrowedRustItem::Struct(item) => &item.id,
            BorrowedRustItem::Enum(item) => &item.shared().id,
            BorrowedRustItem::Alias(item) => &item.id,
            BorrowedRustItem::Const(item) => &item.id,
        }
        .original
        .as_str()
    }
}

/// Given `data`, generate type-code for this language and write it out to `writable`.
/// Returns whether or not writing was successful.
fn generate_types<'c>(
    lang: &impl Language<'c>,
    out: &mut Vec<u8>,
    data: &ParsedData,
    mode: FilesMode<&CrateName>,
) -> anyhow::Result<()> {
    lang.begin_file(out, mode)
        .context("error writing file header")?;

    if let FilesMode::Multi(crate_name) = mode {
        let all_types = HashMap::new();
        lang.write_imports(out, crate_name, used_imports(&data, crate_name, &all_types))
            .context("error writing imports")?;
    }

    let ParsedData {
        structs,
        enums,
        aliases,
        consts,
        ..
    } = data;

    let mut items = Vec::from_iter(
        aliases
            .iter()
            .map(BorrowedRustItem::Alias)
            .chain(structs.iter().map(BorrowedRustItem::Struct))
            .chain(enums.iter().map(BorrowedRustItem::Enum))
            .chain(consts.iter().map(BorrowedRustItem::Const)),
    );

    topsort(&mut items);

    for thing in &items {
        let name = thing.name();

        match thing {
            BorrowedRustItem::Enum(e) => lang
                .write_enum(out, e)
                .with_context(|| format!("error writing enum {name}"))?,
            BorrowedRustItem::Struct(s) => lang
                .write_struct(out, s)
                .with_context(|| format!("error writing struct {name}"))?,
            BorrowedRustItem::Alias(a) => lang
                .write_type_alias(out, a)
                .with_context(|| format!("error writing type alias {name}"))?,
            BorrowedRustItem::Const(c) => lang
                .write_const(out, c)
                .with_context(|| format!("error writing const {name}"))?,
        }
    }

    lang.end_file(out, mode)
        .context("error writing file trailer")
}

/// Lookup any refeferences to other typeshared types in order to build
/// a list of imports for the generated module.
fn used_imports<'a, 'b: 'a>(
    data: &'b ParsedData,
    crate_name: &CrateName,
    all_types: &'a HashMap<CrateName, HashSet<TypeName>>,
) -> BTreeMap<&'a CrateName, BTreeSet<&'a TypeName>> {
    let mut used_imports: BTreeMap<&'a CrateName, BTreeSet<&'a TypeName>> = BTreeMap::new();

    // If we have reference that is a re-export we can attempt to find it with the
    // following heuristic.
    let fallback = |referenced_import: &'a ImportedType,
                    used: &mut BTreeMap<&'a CrateName, BTreeSet<&'a TypeName>>| {
        // Find the first type that does not belong to the current crate.
        if let Some((crate_name, ty)) = all_types
            .iter()
            .flat_map(|(k, v)| {
                v.iter()
                    .find(|&t| *t == referenced_import.type_name && k != crate_name)
                    .map(|t| (k, t))
            })
            .next()
        {
            println!("Warning: Using {crate_name} as module for {ty} which is not in referenced crate {}", referenced_import.base_crate);
            used.entry(crate_name).or_default().insert(ty);
        } else {
            // println!("Could not lookup reference {referenced_import:?}");
        }
    };

    for referenced_import in data
        .import_types
        .iter()
        // Skip over imports that reference the current crate. They
        // are all collapsed into one module per crate.
        .filter(|imp| imp.base_crate != *crate_name)
    {
        // Look up the types for the referenced imported crate.
        if let Some(type_names) = all_types.get(&referenced_import.base_crate) {
            if referenced_import.type_name == "*" {
                // We can have "*" wildcard here. We need to add all.
                used_imports
                    .entry(&referenced_import.base_crate)
                    .and_modify(|names| names.extend(type_names.iter()));
            } else if let Some(ty_name) = type_names.get(&referenced_import.type_name) {
                // Add referenced import for each matching type.
                used_imports
                    .entry(&referenced_import.base_crate)
                    .or_default()
                    .insert(ty_name);
            } else {
                fallback(referenced_import, &mut used_imports);
            }
        } else {
            // We might have a re-export from another crate.
            fallback(referenced_import, &mut used_imports);
        }
    }
    used_imports
}
