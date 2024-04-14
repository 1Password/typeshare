//! Generated source file output.
use crate::args::{ARG_OUTPUT_FILE, ARG_OUTPUT_FOLDER};
use anyhow::Context;
use clap::ArgMatches;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use typeshare_core::{
    language::{CrateName, CrateTypes, Language},
    parser::ParsedData,
    rust_types::RustItem,
};

/// Write the parsed data to the one or more files depending on command line options.
pub fn write_generated(
    options: ArgMatches,
    lang: Box<dyn Language>,
    crate_parsed_data: HashMap<CrateName, ParsedData>,
    import_candidates: CrateTypes,
) -> Result<(), anyhow::Error> {
    let output_folder = options.value_of(ARG_OUTPUT_FOLDER);
    let output_file = options.value_of(ARG_OUTPUT_FILE);

    if let Some(folder) = output_folder {
        write_multiple_files(lang, folder, crate_parsed_data, import_candidates)
    } else if let Some(file) = output_file {
        write_single_file(lang, file, crate_parsed_data)
    } else {
        Ok(())
    }
}

/// Write multiple module files.
fn write_multiple_files(
    mut lang: Box<dyn Language>,
    output_folder: &str,
    crate_parsed_data: HashMap<CrateName, ParsedData>,
    import_candidates: CrateTypes,
) -> Result<(), anyhow::Error> {
    for (_crate_name, parsed_data) in crate_parsed_data {
        let outfile = Path::new(output_folder).join(&parsed_data.file_name);
        let mut generated_contents = Vec::new();
        lang.generate_types(&mut generated_contents, &import_candidates, parsed_data)?;
        check_write_file(&outfile, generated_contents)?;
    }
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

/// Write all types to a single file.
fn write_single_file(
    mut lang: Box<dyn Language>,
    file_name: &str,
    crate_parsed_data: HashMap<CrateName, ParsedData>,
) -> Result<(), anyhow::Error> {
    let mut output = Vec::new();

    let mut sorted_types = crate_parsed_data
        .into_values()
        .flat_map(|parsed_data| {
            parsed_data
                .aliases
                .into_iter()
                .map(RustItem::Alias)
                .chain(parsed_data.structs.into_iter().map(RustItem::Struct))
                .chain(parsed_data.enums.into_iter().map(RustItem::Enum))
        })
        .collect::<Vec<_>>();

    sorted_types.sort_by(|item1, item2| item1.renamed_type_name().cmp(item2.renamed_type_name()));

    let mut parsed_data_combined = ParsedData::default();

    for item in sorted_types {
        match item {
            RustItem::Struct(s) => {
                parsed_data_combined.structs.push(s);
            }
            RustItem::Enum(e) => {
                parsed_data_combined.enums.push(e);
            }
            RustItem::Alias(a) => {
                parsed_data_combined.aliases.push(a);
            }
            t => eprintln!("Unsupported type {t:?}"),
        }
    }

    lang.generate_types(&mut output, &HashMap::new(), parsed_data_combined)?;

    let outfile = Path::new(file_name).to_path_buf();
    check_write_file(&outfile, output)?;
    Ok(())
}
