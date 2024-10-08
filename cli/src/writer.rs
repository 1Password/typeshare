//! Generated source file output.
use anyhow::Context;
use log::info;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
};
use typeshare_core::{
    language::{CrateName, CrateTypes, Language, SINGLE_FILE_CRATE_NAME},
    parser::ParsedData,
};

#[derive(Debug, Clone, Copy)]
pub enum Output<'a> {
    File(&'a Path),
    Folder(&'a Path),
}

/// Write the parsed data to the one or more files depending on command line options.
pub fn write_generated(
    destination: Output<'_>,
    lang: &mut (impl Language + ?Sized),
    crate_parsed_data: BTreeMap<CrateName, ParsedData>,
    import_candidates: CrateTypes,
) -> Result<(), anyhow::Error> {
    match destination {
        Output::File(path) => write_single_file(lang, path, crate_parsed_data),
        Output::Folder(path) => {
            write_multiple_files(lang, path, crate_parsed_data, import_candidates)
        }
    }
}

/// Write multiple module files.
fn write_multiple_files(
    lang: &mut (impl Language + ?Sized),
    output_folder: &Path,
    crate_parsed_data: BTreeMap<CrateName, ParsedData>,
    import_candidates: CrateTypes,
) -> Result<(), anyhow::Error> {
    for (_crate_name, parsed_data) in crate_parsed_data {
        let outfile = Path::new(output_folder).join(&parsed_data.file_name);
        let mut generated_contents = Vec::new();
        lang.generate_types(&mut generated_contents, &import_candidates, parsed_data)?;
        check_write_file(&outfile, generated_contents)?;
    }

    lang.post_generation(&output_folder.as_os_str().to_string_lossy())
        .context("Post generation failed")?;

    Ok(())
}

/// Write the file if the contents have changed.
fn check_write_file(outfile: &Path, output: Vec<u8>) -> anyhow::Result<()> {
    match fs::read(outfile) {
        Ok(buf) if buf == output => {
            // avoid writing the file to leave the mtime intact
            // for tools which might use it to know when to
            // rebuild.
            info!("Skipping writing to {outfile:?} no changes");
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
    lang: &mut (impl Language + ?Sized),
    file_name: &Path,
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
