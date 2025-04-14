//! Generated source file output.
use crate::{args::OutputLocation, topsort::topsort};
use anyhow::Context;
use itertools::Itertools;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use typeshare_model::prelude::*;

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
            let import_candidates = HashMap::new();

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

            write_multiple_files(lang, directory, &crate_parsed_data, &import_candidates)
        }
    }
}

/// Write multiple module files.
pub fn write_multiple_files<'c>(
    lang: &impl Language<'c>,
    output_folder: &Path,
    crate_parsed_data: &HashMap<CrateName, ParsedData>,
    import_candidates: &CrateTypes,
) -> Result<(), anyhow::Error> {
    for (crate_name, parsed_data) in crate_parsed_data {
        let outfile = output_folder.join(&lang.output_file_for_crate(&crate_name));

        let mut output = Vec::new();

        generate_types(
            lang,
            &mut output,
            import_candidates,
            parsed_data,
            FilesMode::Multi(&crate_name),
        )?;

        check_write_file(&outfile, output)?;
    }

    lang.post_generation(output_folder)
        .context("Post generation failed")?;

    Ok(())
}

/// Write all types to a single file.
pub fn write_single_file<'c>(
    lang: &impl Language<'c>,
    file_name: &Path,
    parsed_data: &ParsedData,
) -> Result<(), anyhow::Error> {
    let mut output = Vec::new();

    generate_types(
        lang,
        &mut output,
        &HashMap::new(),
        parsed_data,
        FilesMode::Single,
    )?;

    let outfile = Path::new(file_name).to_path_buf();
    check_write_file(&outfile, output)?;
    Ok(())
}

/// Write the file if the contents have changed.
fn check_write_file(outfile: &PathBuf, output: Vec<u8>) -> anyhow::Result<()> {
    match fs::read(outfile) {
        Ok(buf) if buf == output => {
            // avoid writing the file to leave the mtime intact
            // for tools which might use it to know when to
            // rebuild.
            println!("Skipping writing to {outfile:?} no changes");
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

/// Given `data`, generate type-code for this language and write it out to `writable`.
/// Returns whether or not writing was successful.
fn generate_types<'c>(
    lang: &impl Language<'c>,
    out: &mut Vec<u8>,
    all_types: &CrateTypes,
    data: &ParsedData,
    mode: FilesMode<&CrateName>,
) -> std::io::Result<()> {
    lang.begin_file(out, mode)?;

    if let FilesMode::Multi(crate_name) = mode {
        lang.write_imports(out, crate_name, &used_imports(&data, crate_name, all_types))?;
    }

    let ParsedData {
        structs,
        enums,
        aliases,
        ..
    } = data;

    let mut items = Vec::from_iter(
        aliases
            .iter()
            .map(BorrowedRustItem::Alias)
            .chain(structs.iter().map(BorrowedRustItem::Struct))
            .chain(enums.iter().map(BorrowedRustItem::Enum)),
    );

    topsort(&mut items);

    for thing in &items {
        match thing {
            BorrowedRustItem::Enum(e) => lang.write_enum(out, e)?,
            BorrowedRustItem::Struct(s) => lang.write_struct(out, s)?,
            BorrowedRustItem::Alias(a) => lang.write_type_alias(out, a)?,
            BorrowedRustItem::Const(c) => lang.write_const(out, c)?,
        }
    }

    lang.end_file(out)
}

/// Lookup any refeferences to other typeshared types in order to build
/// a list of imports for the generated module.
fn used_imports<'a, 'b: 'a>(
    data: &'b ParsedData,
    crate_name: &CrateName,
    all_types: &'a CrateTypes,
) -> ScopedCrateTypes<'a> {
    let mut used_imports = ScopedCrateTypes::new();

    // If we have reference that is a re-export we can attempt to find it with the
    // following heuristic.
    let fallback = |referenced_import: &'a ImportedType, used: &mut ScopedCrateTypes<'a>| {
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
