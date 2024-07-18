//! Generated source file output.
use crate::args::{ARG_OUTPUT_FILE, ARG_OUTPUT_FOLDER};
use anyhow::Context;
use clap::ArgMatches;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};
use typeshare_core::{
    language::{CrateName, CrateTypes, Language, SINGLE_FILE_CRATE_NAME},
    parser::ParsedData,
};

/// Write the parsed data to the one or more files depending on command line options.
pub fn write_generated(
    options: ArgMatches,
    lang: Box<dyn Language>,
    crate_parsed_data: BTreeMap<CrateName, ParsedData>,
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
    crate_parsed_data: BTreeMap<CrateName, ParsedData>,
    import_candidates: CrateTypes,
) -> Result<(), anyhow::Error> {
    for (_crate_name, parsed_data) in crate_parsed_data {
        let outfile = Path::new(output_folder).join(&parsed_data.file_name);
        let mut generated_contents = Vec::new();
        lang.generate_types(&mut generated_contents, &import_candidates, parsed_data)?;
        check_write_file(&outfile, generated_contents)?;
    }

    lang.post_generation(output_folder)
        .context("Post generation failed")?;

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
            .with_context(|| format!("Could not get parent for {outfile:?}"))?;
        // If the output directory doesn't already exist, create it.
        if !out_dir.exists() {
            fs::create_dir_all(out_dir).context("failed to create output directory")?;
        }

        fs::write(outfile, output)
            .with_context(|| format!("failed to write output: {}", outfile.to_string_lossy()))?;
    }
    Ok(())
}

/// Write all types to a single file.
fn write_single_file(
    mut lang: Box<dyn Language>,
    file_name: &str,
    mut crate_parsed_data: BTreeMap<CrateName, ParsedData>,
) -> Result<(), anyhow::Error> {
    let parsed_data = crate_parsed_data
        .remove(&SINGLE_FILE_CRATE_NAME)
        .context("Could not get parsed data for single file output")?;

    let mut output = Vec::new();
    lang.generate_types(&mut output, &HashMap::new(), parsed_data)?;

    let outfile = Path::new(file_name).to_path_buf();
    check_write_file(&outfile, output)?;
    Ok(())
}
