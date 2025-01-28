//! Generated source file output.
use crate::parser::SINGLE_FILE_CRATE_NAME;
use anyhow::Context;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use typeshare_model::prelude::*;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum OutputMode<'a> {
    SingleFile {
        file: &'a Path,
    },
    MultiFile {
        folder: &'a Path,
        import_candidates: &'a CrateTypes,
    },
}

/// Write the parsed data to the one or more files depending on command line options.
pub fn write_generated(
    output: OutputMode<'_>,
    lang: &mut impl Language,
    crate_parsed_data: HashMap<CrateName, ParsedData>,
) -> Result<(), anyhow::Error> {
    match output {
        OutputMode::MultiFile {
            folder,
            import_candidates,
        } => write_multiple_files(lang, folder, crate_parsed_data, import_candidates),
        OutputMode::SingleFile { file } => write_single_file(lang, file, crate_parsed_data),
    }
}

/// Write multiple module files.
fn write_multiple_files(
    lang: &mut impl Language,
    output_folder: &Path,
    crate_parsed_data: HashMap<CrateName, ParsedData>,
    import_candidates: &CrateTypes,
) -> Result<(), anyhow::Error> {
    for (_crate_name, parsed_data) in crate_parsed_data {
        let outfile = Path::new(output_folder).join(&parsed_data.file_name);
        let mut generated_contents = Vec::new();
        lang.generate_types(
            &mut generated_contents,
            import_candidates,
            parsed_data,
            FilesMode::Multi,
        )?;
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
    lang: &mut impl Language,
    file_name: &Path,
    mut crate_parsed_data: HashMap<CrateName, ParsedData>,
) -> Result<(), anyhow::Error> {
    let parsed_data = crate_parsed_data
        .remove(&SINGLE_FILE_CRATE_NAME)
        .context("Could not get parsed data for single file output")?;

    let mut output = Vec::new();
    lang.generate_types(&mut output, &HashMap::new(), parsed_data, FilesMode::Single)?;

    let outfile = Path::new(file_name).to_path_buf();
    check_write_file(&outfile, output)?;
    Ok(())
}
