use std::{ffi::CStr, path::PathBuf};

use anyhow::Context;
use clap::builder::PossibleValuesParser;
use itertools::Itertools;
use serde::{ser, Deserialize, Serialize};
use typeshare_model::cli::{ArgType, ConfigCliArgs};
use typeshare_model::{language, Language};

use crate::config::Config;

/// Given a language type, use the name of the language and information on its
/// config type to populate a clap command with CLI
fn add_language_params_to_clap<'a, L: Language<'a>>(command: clap::Command) -> clap::Command {
    if let Some(arg) = command
        .get_arguments()
        .find(|arg| arg.get_id().as_str().starts_with(L::NAME))
    {
        panic!(
            "existing argument {:?} conflicts with language {}",
            arg.get_id().as_str(),
            L::NAME,
        )
    }

    L::Config::ARGS
        .iter()
        .fold(command, |command, (arg_name, arg_type)| {
            let name = format!("{language}-{flag}", language = L::NAME, flag = arg_name);

            let arg = clap::Arg::new(name.clone()).long(name).required(false);

            command.arg(match arg_type {
                ArgType::Switch => arg.action(clap::ArgAction::SetTrue),
                ArgType::Value => arg.action(clap::ArgAction::Set).value_name(arg_name),
            })
        })
}

/// Given a config object for a language (populated from a config file),
/// populate it further from configuration from the CLI, based on an ArgsSet
/// specification.
///
/// The way this works is slightly magical, but basically we serialize the
/// object into generic nested key-value data, then re-deserialize it,
/// injecting values from the arg matches as necessary.
fn populate_config_object_from_cli_matches<'config, C: ConfigCliArgs<'config>>(
    mut config: C,
    matches: &'config clap::ArgMatches,
    language: &str,
) -> anyhow::Result<C> {
    use std::fmt::Write as _;

    // TODO: currently this bails on the first error. It would be nice if it
    // could collect all related errors

    let mut full_name = String::new();

    for &(name, arg_type) in C::ARGS {
        full_name.clear();
        write!(&mut full_name, "{language}-{name}");

        match arg_type {
            ArgType::Switch => {
                if matches.get_flag(&full_name) {
                    config.enable_cli_switch(name);
                }
            }
            ArgType::Value => {
                if let Ok(Some(args)) = matches.try_get_raw(&full_name) {
                    if let Ok(arg) = args.exactly_one() {
                        if let Some(arg) = arg.to_str() {
                            config.apply_cli_value(name, arg)?;
                        }
                    }
                }
            }
        }
    }

    Ok(config)
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
pub struct Output {
    /// The file to write all typeshare output to. When this flag is used, all
    /// typeshare output will be into a single file. This it the typical way
    /// typeshare is used.
    #[arg(short, long)]
    pub output_file: Option<PathBuf>,

    /// The directory into which to write all typeshare output. When this flag
    /// is used, typeshare will emit one source file *per crate* from the
    /// scanned rust code. Multi-file support is still somewhat unstable while
    /// certain bugs are resolved.
    #[arg(short = 'd', long)]
    pub ouput_directory: Option<PathBuf>,
}

#[derive(clap::Parser)]
pub struct StandardArgs {
    /// Path to the config file for this typeshare
    #[arg(short, long, visible_alias("config-file"))]
    pub config: Option<PathBuf>,

    #[command(flatten)]
    pub output: Output,

    /// The directories within which to recursively find and process rust files
    #[arg(num_args(1..))]
    pub directories: Vec<PathBuf>,
}

pub fn personalize_args(
    command: clap::Command,
    name: &'static str,
    version: &'static str,
    author: &'static str,
    about: &'static str,
) -> clap::Command {
    command
        .name(name)
        .version(version)
        .author(author)
        .about(about)
}

/// Add a `--lang` argument to the command. This argument will be optional if
/// there is only one language
pub fn add_lang_argument(command: clap::Command, languages: &[&'static str]) -> clap::Command {
    let arg = clap::Arg::new("language")
        .short('l')
        .long("lang")
        .value_name("LANGUAGE")
        .value_parser(PossibleValuesParser::new(languages))
        .action(clap::ArgAction::Set)
        .help("the output language of generated types");

    command.arg(match languages {
        [] => panic!("need at least one language"),
        [lang] => arg.required(false).default_value(lang),
        _ => arg.required(true),
    })
}
