use crate::cli::config::{find_configuration_file, store_config, Config};
use crate::language::{Language, LanguageConfig};
pub use clap;
use clap::{Args, Parser, Subcommand};
pub use clap_complete;
pub use ignore;
use std::fs::File;
use std::io::{read_to_string, Error};
use std::path::{Path, PathBuf};
use std::{io, process};
pub use toml;

pub mod config;

pub fn init_config<L: Language>(config: impl AsRef<Path>) -> ! {
    let config_path = config.as_ref();
    if config_path.exists() {
        println!("Config file already exists");
        process::exit(1);
    }
    add_lang_to_config::<L>(config_path);
}
pub fn load_config_optional_path<L: Language, P: AsRef<Path>>(
    config: Option<P>,
) -> Result<(Config, L::Config), io::Error> {
    match config {
        Some(config) => load_config::<L>(config),
        None => {
            let config = find_configuration_file().unwrap_or(PathBuf::from("typeshare.toml"));
            load_config::<L>(config)
        }
    }
}
pub fn load_config<L: Language>(
    config: impl AsRef<Path>,
) -> Result<(Config, L::Config), io::Error> {
    let config = config.as_ref();
    let mut config: Config = if config.exists() {
        toml::from_str(read_to_string(File::open(config)?)?.as_str()).map_err(|v| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Could not parse config: {}", v),
            )
        })?
    } else {
        return Ok((Config::default(), L::Config::default()));
    };

    return match config.language.remove(L::language_name()) {
        None => Ok((config, L::Config::default())),
        Some(some) => {
            let lang_config = some.try_into::<L::Config>().map_err(|v| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Could not parse language config: {} for {}",
                        v,
                        L::language_name()
                    ),
                )
            })?;
            Ok((config, lang_config))
        }
    };
}
pub fn add_lang_to_config<L: Language>(config_path: impl AsRef<Path>) -> ! {
    let (config, lang_config) = match load_config::<L>(&config_path) {
        Ok((config, lang_config)) => (config, lang_config),
        Err(err) => {
            eprintln!("Could not load config file: {}", err);
            process::exit(1);
        }
    };
    let mut config = config;
    config.language.insert(
        L::language_name().to_string(),
        toml::Value::try_from(&lang_config).unwrap(),
    );
    match config::store_config(&config, Some(&config_path)) {
        Ok(_) => {
            println!("Config written to {}", config_path.as_ref().display());
            process::exit(0);
        }
        Err(err) => {
            eprintln!("Could not write config file: {}", err);
            process::exit(1);
        }
    };
}
#[macro_export]
macro_rules! define_command {
    ($args:ty, $lang:ty) => {
        use typeshare_core::cli::clap;
        #[derive(clap::Parser)]
        pub struct Command {
            #[clap(subcommand)]
            pub subcommand: Commands,
        }
        #[derive(clap::Parser)]
        pub struct GenerateCommand {
            #[clap(long, short)]
            pub config: Option<std::path::PathBuf>,
            #[clap(long, short)]
            pub output: std::path::PathBuf,
            #[clap(flatten)]
            pub command: $args,
            pub directories: Vec<String>,
        }
        impl GenerateCommand {
            pub fn load_or_exit(
                self,
            ) -> (
                typeshare_core::cli::config::Config,
                <$lang as typeshare_core::language::Language>::Config,
                std::path::PathBuf,
            ) {
                match typeshare_core::cli::load_config_optional_path::<$lang, _>(self.config) {
                    Ok((mut core_config, mut lang_config)) => {
                        lang_config = lang_config + self.command;
                        core_config.provided_directories(self.directories);
                        return (core_config, lang_config, self.output);
                    }
                    Err(err) => {
                        eprintln!("Could not load config file: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
        #[derive(clap::Subcommand)]
        #[non_exhaustive]
        pub enum Commands {
            Init {
                #[clap(long, short)]
                config: Option<std::path::PathBuf>,
            },
            /// Adds a language to the config file
            AddLangToConfig {
                #[clap(long, short)]
                config: Option<std::path::PathBuf>,
            },
            /// You can use this command without calling the subcommand name
            Generate(GenerateCommand),
            TypeShareInfo,
        }
        impl Commands {
            pub fn parse_and_handle_extras() -> GenerateCommand {
                let result = <Command as clap::Parser>::parse();

                match result.subcommand {
                    Commands::Init { config } => {
                        let config = config.unwrap_or(std::path::PathBuf::from("typeshare.toml"));
                        typeshare_core::cli::init_config::<$lang>(config);
                    }
                    Commands::AddLangToConfig { config } => {
                        let config = config.unwrap_or(std::path::PathBuf::from("typeshare.toml"));
                        typeshare_core::cli::add_lang_to_config::<$lang>(config);
                    }
                    Commands::Generate(generate_command) => {
                        return generate_command;
                    }
                    Commands::TypeShareInfo => {
                        println!("TypeShare Version: {}", env!("CARGO_PKG_VERSION"));
                        std::process::exit(0);
                    }
                    _ => {
                        unreachable!("This should never happen");
                    }
                }
            }
        }
    };
}
pub use define_command;
