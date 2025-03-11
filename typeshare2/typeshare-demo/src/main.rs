use typeshare_driver::typeshare_binary;
use typeshare_typescript::TypeScript;

// // Recursive expansion of typeshare_binary! macro
// // ===============================================

// fn main() -> ::anyhow::Result<()> {
//     use ::clap::{CommandFactory as _, FromArgMatches as _, Parser as _};
//     use ::ignore::{types::TypesBuilder, WalkBuilder};
//     use ::std::collections::HashMap;
//     use ::typeshare_engine::args::add_language_params_to_clap;
//     use ::typeshare_engine::config::{compute_args_set, load_language_config_from_file};
//     use ::typeshare_model::Language as LanguageTrait;
//     #[allow(non_snake_case)]
//     struct LangArgsSets {
//         TypeScript: CliArgsSet,
//     }
//     let language_metas = LangArgsSets {
//         TypeScript: compute_args_set::<TypeScript>(),
//     };
//     let command = StandardArgs::command();
//     let command = command
//         .name(personalize.name)
//         .version(personalize.version)
//         .author(personalize.author)
//         .about(personalize.about);
//     let command = ::typeshare::engine::args::add_lang_argument(command, &[TypeScript::NAME]);
//     let command = add_language_params_to_clap(command, language_metas.TypeScript);
//     let args = command.get_matches();
//     let standard_args = StandardArgs::from_arg_matches(&args)
//         .expect("StandardArgs should always be loadable from a `command`");
//     let config = ::typeshare_engine::config::load_config(standard_args.config.as_deref())?;
//     let walker = {
//         let directories = standard_args.directories.as_slice();
//         let (first_dir, other_dirs) = directories.split_first().unwrap();
//         let mut types = TypesBuilder::new();
//         types.add("rust", "*.rs").unwrap();
//         types.select("rust");
//         let mut walker_builder = WalkBuilder::new(first_dir);
//         walker_builder.types(types.build().unwrap());
//         other_dirs.iter().for_each(|dir| {
//             walker_builder.add(dir);
//         });
//         walker_builder.build()
//     };
//     let parser_inputs = parser_inputs(walker_builder);
//     let data = parse_input(
//         parser_inputs,
//         &[],
//         if standard_args.output.output_file.is_some() {
//             FilesMode::Single
//         } else {
//             FilesMode::Multi
//         },
//     )
//     .unwrap();
//     let language: &str = args
//         .get_one("language")
//         .expect("clap should guarantee that --lang is provided");
//     return if false {
//     } else if language == <TypeScript as LanguageTrait>::NAME {
//         let name = <TypeScript as LanguageTrait>::NAME;
//         let config = load_language_config_from_file(&config, &args, &language_metas.TypeScript)
//             .with_context(|| {
//                 alloc::__export::must_use({
//                     let res = alloc::fmt::format(alloc::__export::format_args!(
//                         "failed to load configuration for language {name}"
//                     ));
//                     res
//                 })
//             })?;
//         let language_implementation =
//             <TypeScript>::new_from_config(&config).with_context(|| {
//                 alloc::__export::must_use({
//                     let res = alloc::fmt::format(alloc::__export::format_args!(
//                         "failed to load configuration for language {name}"
//                     ));
//                     res
//                 })
//             })?;
//     } else {
//         core::panicking::panic_fmt(core::const_format_args!(
//             "Unrecognized language {language}; clap should have prevented this"
//         ));
//     };
// }

typeshare_binary! { TypeScript }
