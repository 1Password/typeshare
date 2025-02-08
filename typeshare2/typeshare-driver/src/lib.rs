#[macro_export]
macro_rules! typeshare_binary {
    ($($Language:ident),+ $(,)?) => {
        fn main() {
            use clap::Parser;
            use clap::{CommandFactory, FromArgMatches};
            use ignore::{types::TypesBuilder, WalkBuilder};
            use std::collections::HashMap;
            use typeshare_engine::{
                args::{add_lang_argument, personalize_args, Output, StandardArgs},
                parser::{parse_input, parser_inputs},
                FilesMode,
            };

            let command = StandardArgs::command();
            let command = personalize_args(
                command,
                "typeshare-typescript",
                "1.0",
                "1Password",
                "A typeshare that generates typescript code",
            );
            let command = add_lang_argument(command, &["typescript", "swift"]);
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

          $crate::use_language! {
                config, lang, {}; args.get_one("language").unwrap_or(&"typescript"); $($Language,)+
            }
        }
    };
}

#[macro_export]
macro_rules! use_language {
    ($config: expr, $lang:ident, $op:expr; $switch:expr; $($Language:ident,)+) => {
        {
            use typeshare_model::Language;

            let choice = $switch;
            let config = $config;

            if false {}
            $(
               else if *choice == $Language::NAME {
                    let $lang = $Language::new_from_config(config.config_for_language(choice).unwrap()).unwrap();
                    $op
                }
            )+
            else { panic!("OOPS") }
        }
    };
}
