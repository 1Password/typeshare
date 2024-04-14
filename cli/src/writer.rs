use crate::args::{ARG_OUTPUT_FILE, ARG_OUTPUT_FOLDER};
use anyhow::Context;
use clap::ArgMatches;
use std::{
    collections::HashMap,
    fs,
    ops::ControlFlow,
    path::{Path, PathBuf},
};
use typeshare_core::{
    language::{CrateName, CrateTypes, Language},
    parser::ParsedData,
};

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
        if let ControlFlow::Break(_) = check_write_file(&outfile, generated_contents)? {
            continue;
        }
    }
    Ok(())
}

fn check_write_file(outfile: &PathBuf, output: Vec<u8>) -> anyhow::Result<ControlFlow<()>> {
    match fs::read(outfile) {
        Ok(buf) if buf == output => {
            // avoid writing the file to leave the mtime intact
            // for tools which might use it to know when to
            // rebuild.
            println!("Skipping writing to {outfile:?} no changes");
            return Ok(ControlFlow::Break(()));
        }
        _ => {}
    }

    if !output.is_empty() {
        let out_dir = outfile.parent().unwrap();
        // If the output directory doesn't already exist, create it.
        if !out_dir.exists() {
            fs::create_dir_all(out_dir).context("failed to create output directory")?;
        }

        fs::write(outfile, output).context("failed to write output")?;
    }
    Ok(ControlFlow::Continue(()))
}

fn write_single_file(
    mut lang: Box<dyn Language>,
    file_name: &str,
    crate_parsed_data: HashMap<CrateName, ParsedData>,
) -> Result<(), anyhow::Error> {
    let mut output = Vec::new();
    for data in crate_parsed_data.into_values() {
        lang.generate_types(&mut output, &HashMap::new(), data)?;
    }

    let outfile = Path::new(file_name).to_path_buf();
    check_write_file(&outfile, output)?;
    Ok(())
}
