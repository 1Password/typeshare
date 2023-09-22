//! This is the command line tool for Typeshare. It is used to generate source files in other
//! languages based on Rust code.

mod config;
mod ffi;

use std::collections::VecDeque;
use std::convert::Infallible;
use std::path::PathBuf;

use crate::config::CLIConfig;
use clap::{arg, command, ArgMatches, Command, CommandFactory, FromArgMatches, Parser, Subcommand};
use log::{error, warn};
use syntect::highlighting::{ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxSetBuilder};
use tabled::Table;
use thiserror::Error;
use toml::Value;
use typeshare_core::cli::config::{load_config, Config as ProjectConfig};
static TOML_SYNTAX: &str =
    include_str!("../highlighting/sublime_toml_highlighting/TOML.sublime-syntax");
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    FFIError(#[from] ffi::Error),
    #[error("Could not serialize config file: {0}. This is a bug, please report it.")]
    SerializeTomlError(#[from] toml::ser::Error),
    #[error("Could not parse config file: {0}")]
    DeserializeTomlError(#[from] toml::de::Error),
}
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    subcommand: GenericSubCommands,
}
#[derive(Subcommand, Clone)]
pub enum GenericSubCommands {
    #[clap(about = "Initializes the project configuration file")]
    Init {
        #[clap(long, short)]
        config: Option<PathBuf>,
        #[clap(help = "Languages to add to the config for default values")]
        langs: Vec<String>,
    },
    LangConfig {
        #[clap(long, short)]
        lang: String,
        #[clap(long, short, default_value = "true")]
        pretty: bool,
    },
    #[clap(about = "Checks the Project for Typeshare Macro Errors")]
    Check {
        #[clap(long, short)]
        config: Option<PathBuf>,
        directories: Vec<String>,
    },
    Info,
    LoadLanguages,
}
fn main() {
    typeshare_core::cli::init_log(); // Better Logging
    let app_config = match CLIConfig::get_app_config() {
        Ok(ok) => ok,
        Err(err) => {
            error!("Could not load config file: {}", err);
            std::process::exit(1);
        }
    };
    let command_cli = CLI::command_for_update()
        .subcommand(
            Command::new("generate")
                .about("Parses Rust code and generates types in other languages")
                .arg(arg!(
                    <LANG> "The language to generate"
                )),
        )
        .subcommand_required(true);
    let mut matches = command_cli.get_matches();
    let result = if matches.subcommand_matches("generate").is_some() {
        let (_, arguments) = matches.remove_subcommand().unwrap();
        generate(arguments)
    } else {
        let sub_command = match CLI::from_arg_matches(&matches) {
            Ok(ok) => ok.subcommand,
            Err(err) => err.exit(),
        };
        execute_normal(sub_command, app_config)
    };
    if let Err(err) = result {
        error!("{}", err);
        std::process::exit(1);
    }
}
fn generate(matches: ArgMatches) -> Result<(), Error> {
    println!("Generating");
    println!("{:?}", matches);
    std::process::exit(0);
}
fn execute_normal(cli: GenericSubCommands, mut cli_config: CLIConfig) -> Result<(), Error> {
    match cli {
        GenericSubCommands::Init { config, langs } => {
            let config = config.unwrap_or(PathBuf::from("typeshare.toml"));
            if config.exists() {
                error!("Config file already exists");
                return Ok(());
            }
            let mut project_config = ProjectConfig::default();

            for lang in langs {
                let Some(lang) = cli_config.get_language(&lang) else {
                    warn!("Could not find language {}", lang);
                    continue;
                };
                if let Some(lang_default_value) = &lang.default_config {
                    // TODO move to toml_edit to keep comments
                    let value = toml::from_str::<Value>(lang_default_value)?;
                    project_config
                        .language
                        .insert(lang.language.name().to_string(), value);
                } else {
                    warn!(
                        "Could not find default config for language {}",
                        lang.language.name()
                    );
                }
            }
            let toml = toml::to_string_pretty(&project_config)?;
            std::fs::write(config, toml)?;
        }
        GenericSubCommands::LangConfig { lang, pretty } => {
            let Some(lang) = cli_config.get_language(&lang) else {
                warn!("Could not find language {}", lang);
                return Ok(());
            };
            let Some(config) = lang.default_config.as_ref() else {
                warn!(
                    "Could not find default config for language {}",
                    lang.language.name()
                );
                return Ok(());
            };
            if pretty {
                let mut ps = SyntaxSetBuilder::new();
                ps.add(SyntaxDefinition::load_from_str(TOML_SYNTAX, true, None).unwrap());
                let ps = ps.build();
                let ts = ThemeSet::load_defaults();
                let syntax = ps.find_syntax_by_extension("toml").unwrap();
                let mut highlighter =
                    syntect::easy::HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
                for line in config.lines() {
                    match highlighter.highlight_line(line, &ps) {
                        Ok(ranges) => {
                            let escaped =
                                syntect::util::as_24_bit_terminal_escaped(&ranges[..], false);
                            println!("{}", escaped)
                        }
                        Err(err) => {
                            error!("Could not highlight line: {}", err);
                            return Ok(());
                        }
                    }
                }
            } else {
                println!("{}", config);
            }
        }
        GenericSubCommands::LoadLanguages => {
            cli_config.rebuild_languages()?;
            cli_config.save()?;
            println!("Languages loaded");
            let table = Table::new(cli_config.languages.iter().map(|v| &v.language));
            println!("{}", table);
        }
        GenericSubCommands::Info => {}
        GenericSubCommands::Check {
            config,
            directories,
        } => {
            return check(config, directories);
        }
    }
    Ok(())
}

pub fn check(config: Option<PathBuf>, directories: Vec<String>) -> Result<(), Error> {
    let config = config.unwrap_or(PathBuf::from("typeshare.toml"));
    let project_config = load_config(config)?;
    let directories = if !directories.is_empty() {
        VecDeque::from(directories)
    } else {
        project_config.directories
    };
    let result = typeshare_core::generate_parse::<Infallible>(directories);
    match result {
        Ok(_) => {
            println!("Success!");
            std::process::exit(0);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}
