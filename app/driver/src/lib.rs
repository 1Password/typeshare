#[doc(hidden)]
pub mod reexport {
    pub use typeshare_engine as engine;
    pub use typeshare_model as model;

    pub use anyhow;
    pub use clap;
    pub use clap_complete;
    pub use ignore;
}

#[macro_export]
macro_rules! typeshare_binary {
    ($($Language:ident),+ $(,)?) => {
        fn main() -> $crate::reexport::anyhow::Result<()> {
            use ::std::{
                collections::HashMap,
                io,
            };

            use $crate::reexport::{
                anyhow::Context as _,
                clap::{CommandFactory as _, FromArgMatches as _, Parser as _},
                engine::{
                    args::{add_language_params_to_clap, StandardArgs, add_lang_argument, Command},
                    config::{compute_args_set, load_language_config_from_file, CliArgsSet, load_config},
                    parser::{parse_input, parser_inputs},
                    writer::write_output,
                },
                ignore::{types::TypesBuilder, WalkBuilder},
                model::{prelude::FilesMode, Language as LanguageTrait},
                clap_complete::generate as generate_completions,
            };

            #[allow(non_snake_case)]
            struct LangArgsSets {
                $($Language : CliArgsSet,)*
            }

            let language_metas = LangArgsSets {
                $($Language: compute_args_set::<$Language>()?,)*
            };

            let command = StandardArgs::command();

            // let command = command
            //     .name(personalize.name)
            //     .version(personalize.version)
            //     .author(personalize.author)
            //     .about(personalize.about);

            let command = add_lang_argument(command, &[$($Language::NAME,)*]);

            $(
                let command = add_language_params_to_clap(command, &language_metas.$Language);
            )*

            // Parse command line arguments. Need to clone here because we
            // need to be able to generate completions later.
            let args = command.clone().get_matches();

            // Load the standard arguments from the parsed arguments. Generally
            // we expect that this won't fail, because the `command` has been
            // configured to only give us valid arrangements of args
            let standard_args = StandardArgs::from_arg_matches(&args)
                .expect("StandardArgs should always be loadable from a `command`");

            // If we asked for completions, do that before anything else
            if let Some(options) = standard_args.subcommand {
                match options {
                    Command::Completions { shell } => {
                        let mut command = command;
                        let bin_name = command.get_name().to_string();
                        generate_completions(shell, &mut command, bin_name, &mut io::stdout());
                    }
                }

                return Ok(());
            }

            // Load all of the language configurations
            let config = load_config(standard_args.config.as_deref())?;

            // Construct the directory walker that will produce the list of
            // files to typeshare
            let walker = {
                let directories = standard_args.directories.as_slice();
                let (first_dir, other_dirs) = directories.split_first().expect(
                    "clap should guarantee that there's at least one output directory"
                );

                let mut types = TypesBuilder::new();
                types.add("rust", "*.rs").unwrap();
                types.select("rust");

                let mut walker_builder = WalkBuilder::new(first_dir);
                walker_builder.types(types.build().unwrap());
                other_dirs.iter().for_each(|dir| {
                    walker_builder.add(dir);
                });
                walker_builder.build()
            };

            // Collect all of the files we intend to parse with typeshare
            let parser_inputs = parser_inputs(walker);

            // Parse those files
            let data = parse_input(
                parser_inputs,
                &[],
                if standard_args.output.file.is_some() {
                    FilesMode::Single
                } else {
                    FilesMode::Multi(())
                },
            )
            .unwrap();

            let destination = standard_args.output.location();

            let language: &String = args
                .get_one("language")
                .expect("clap should guarantee that --lang is provided");

            return if false {unreachable!()}
            $(
                else if language == <$Language as LanguageTrait>::NAME {
                    let name = <$Language as LanguageTrait>::NAME;

                    let config = load_language_config_from_file::<$Language>(
                        &config,
                        &args,
                        &language_metas.$Language,
                    ).with_context(|| format!(
                        "failed to load configuration for language {name}"
                    ))?;

                    let language_implementation = <$Language>::new_from_config(config)
                        .with_context(|| format!("failed to load configuration for language {name}"))?;

                    write_output(&language_implementation, data, destination)
                        .with_context(|| format!("failed to generate typeshared code for language {name}"))?;

                    Ok(())
                }
            )*
            else {
                panic!("Unrecognized language {language}; clap should have prevented this")
            }
        }
    };
}
