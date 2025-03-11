#[macro_export]
macro_rules! typeshare_binary {
    ($($Language:ident),+ $(,)?) => {
        fn main() -> ::anyhow::Result<()> {
            use ::std::collections::HashMap;

            use ::clap::{Parser as _, CommandFactory as _, FromArgMatches as _};
            use ::ignore::{types::TypesBuilder, WalkBuilder};
            use ::anyhow::Context as _;

            use ::typeshare_model::Language as LanguageTrait;
            use ::typeshare_model::prelude::FilesMode;
            use ::typeshare_engine::config::{compute_args_set, load_language_config_from_file, CliArgsSet};
            use ::typeshare_engine::args::{add_language_params_to_clap, StandardArgs};
            use ::typeshare_engine::writer::write_output;
            use ::typeshare_engine::parser::{parser_inputs, parse_input};

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

            let command = ::typeshare_engine::args::add_lang_argument(command, &[$($Language::NAME,)*]);

            $(
                eprintln!("{:#?}", &language_metas.$Language);
                let command = add_language_params_to_clap(command, &language_metas.$Language);
            )*

            eprintln!("{command:#?}");

            // Parse command line arguments.
            let args = command.get_matches();

            eprintln!("{args:#?}");

            // Load the standard arguments from the parsed arguments. Generally
            // we expect that this won't fail, because the `command` has been
            // configured to only give us valid arrangements of args
            let standard_args = StandardArgs::from_arg_matches(&args)
                .expect("StandardArgs should always be loadable from a `command`");

            eprintln!("{standard_args:#?}");

            // Load all of the language configurations
            let config = ::typeshare_engine::config::load_config(standard_args.config.as_deref())?;

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
                if standard_args.output.output_file.is_some() {
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
