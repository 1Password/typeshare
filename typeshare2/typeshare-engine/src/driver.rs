use clap::CommandFactory as _;

use crate::args::{add_lang_argument, StandardArgs};

pub struct PersonalizeClap {
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub about: &'static str,
}

fn main_skeleton<CliArgSets>(
    personalize: &PersonalizeClap,
    languages: &[&'static str],
    compute_cli_arg_sets: impl FnOnce() -> CliArgSets,
    apply_cli_arg_sets_to_command: impl FnOnce(clap::Command, &CliArgSets) -> clap::Command,
) {
    // use clap::Parser;
    // use clap::{CommandFactory, FromArgMatches};
    // use ignore::{types::TypesBuilder, WalkBuilder};
    // use std::collections::HashMap;
    // use typeshare_engine::{
    //     args::{add_lang_argument, personalize_args, Output, StandardArgs},
    //     parser::{parse_input, parser_inputs},
    //     FilesMode,
    // };

    let command = StandardArgs::command();

    let command = command
        .name(personalize.name)
        .version(personalize.version)
        .author(personalize.author)
        .about(personalize.about);

    let command = add_lang_argument(command, languages);

    let args = command.get_matches();

    let standard_args = StandardArgs::from_arg_matches(&args).unwrap();

    let config = typeshare_engine::config::load_config(standard_args.config.as_deref()).unwrap();
    let directories = standard_args.directories.as_slice();
    let (first_dir, other_dirs) = directories.split_first().unwrap();

    let mut types = TypesBuilder::new();
    types.add("rust", "*.rs").unwrap();
    types.select("rust");

    let mut walker_builder = WalkBuilder::new(first_dir);
    walker_builder.types(types.build().unwrap());
    other_dirs.iter().for_each(|dir| {
        walker_builder.add(dir);
    });

    let parser_inputs = parser_inputs(walker_builder.build());
    let data = parse_input(
        parser_inputs,
        &[],
        if standard_args.output.output_file.is_some() {
            FilesMode::Single
        } else {
            FilesMode::Multi
        },
    )
    .unwrap();
}
