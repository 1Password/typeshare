use crate::args::{ARG_OUTPUT_FILE, ARG_OUTPUT_FOLDER};
use clap::ArgMatches;
use std::{collections::HashMap, fs, path::Path};
use typeshare_core::{
    language::{CrateName, CrateTypes, Language},
    parser::ParsedData,
};

pub fn write_generated(
    options: ArgMatches,
    lang: Box<dyn Language>,
    crate_parsed_data: HashMap<CrateName, ParsedData>,
    import_candidates: CrateTypes,
) {
    let output_folder = options.value_of(ARG_OUTPUT_FOLDER);
    let output_file = options.value_of(ARG_OUTPUT_FILE);

    if let Some(folder) = output_folder {
        write_multiple_files(lang, folder, crate_parsed_data, import_candidates);
    } else if let Some(file) = output_file {
        write_single_file(lang, file, crate_parsed_data);
    }
}

fn write_multiple_files(
    mut lang: Box<dyn Language>,
    output_folder: &str,
    crate_parsed_data: HashMap<CrateName, ParsedData>,
    import_candidates: CrateTypes,
) {
    for (_crate_name, parsed_data) in crate_parsed_data {
        // Print any errors
        for error in &parsed_data.errors {
            eprintln!(
                "Failed to parse {} for crate {} {}",
                error.file_name, error.crate_name, error.error
            );
        }

        let outfile = Path::new(output_folder).join(&parsed_data.file_name);
        let mut generated_contents = Vec::new();
        lang.generate_types(&mut generated_contents, &import_candidates, &parsed_data)
            .expect("Couldn't generate types");
        match fs::read(&outfile) {
            Ok(buf) if buf == generated_contents => {
                // ok! don't need to do anything :)
                // avoid writing the file to leave the mtime intact
                // for tools which might use it to know when to
                // rebuild.
                println!("Skipping writing to {outfile:?} no changes");
                continue;
            }
            _ => {}
        }

        if !generated_contents.is_empty() {
            let out_dir = outfile.parent().unwrap();
            // If the output directory doesn't already exist, create it.
            if !out_dir.exists() {
                fs::create_dir_all(out_dir).expect("failed to create output directory");
            }

            fs::write(outfile, generated_contents).expect("failed to write output");
        }
    }
}

fn write_single_file(
    mut lang: Box<dyn Language>,
    file_name: &str,
    crate_parsed_data: HashMap<CrateName, ParsedData>,
) {
}
