use anyhow::Context;
use serde::{Deserialize, Serialize};
use typeshare_model::Language;
use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Path, PathBuf},
};

use crate::serde::{args::ArgsSet, EmptyDeserializer};

const DEFAULT_CONFIG_FILE_NAME: &str = "typeshare.toml";

// /// The paramters that are used to configure the behaviour of typeshare
// /// from the configuration file `typeshare.toml`
// #[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
// #[serde(default)]
// pub(crate) struct Config {
//     pub swift: SwiftParams,
//     pub typescript: TypeScriptParams,
//     pub kotlin: KotlinParams,
//     pub scala: ScalaParams,
//     #[cfg(feature = "go")]
//     pub go: GoParams,
// }

// TODO: someday we'd like to support borrowed data here. For now, though, it
// seems as though there's no support for borrowed data in the `toml` crate,
// so it would be wasted effort.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Config {
    raw_data: BTreeMap<String, toml::Table>,
}

impl Config {
    /// Retrieve the config for the given language, by deserializing it into
    /// the given type. The deserialize implementation should be able to handle
    /// arbitrary missing keys by populating them with default values. Errors
    ///
    pub fn config_for_language(&self, language: &str) -> Option<&toml::Table>

    {
        self.raw_data.get(language)
    }

    // Store a config for a language, overriding the existing one, by
    // serializing the config type into a toml table
    pub fn store_config_for_language<T>(&mut self, language: &str, config: &T) {
        todo!()
    }
}

// pub fn store_config(config: &Config, file_path: Option<&str>) -> anyhow::Result<()> {
//     let file_path = file_path.unwrap_or(DEFAULT_CONFIG_FILE_NAME);
//     let config_output = toml::to_string_pretty(config).context("Failed to serialize to toml")?;

//     // Fail if trying to overwrite an existing config file
//     let mut file = OpenOptions::new()
//         .write(true)
//         .create_new(true)
//         .open(file_path)?;

//     file.write_all(config_output.as_bytes())?;

//     Ok(())
// }

pub fn load_config(file_path: Option<&Path>) -> anyhow::Result<Config> {
    let file_path_buf;

    let file_path = match file_path {
        Some(path) => path,
        None => match find_configuration_file() {None=>return Ok(Config::default()),
            Some(path) => {
                file_path_buf = path;
                &file_path_buf
            }
        }
    };

    let config_string = fs::read_to_string(file_path).
    if let Some(file_path) = file_path.or_else(|| find_configuration_file().as_deref()) {
        let config_string = fs::read_to_string(file_path)?;
        toml::from_str(&config_string).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    } else {
        Ok(Config::default())
    }
}

/// Search each ancestor directory for configuration file
fn find_configuration_file() -> Option<PathBuf> {
    let mut path = env::current_dir().ok()?;
    let file = Path::new(DEFAULT_CONFIG_FILE_NAME);

    loop {
        path.push(file);

        if path.is_file() {
            break Some(path);
        } else if !(path.pop() && path.pop()) {
            break None;
        }
    }
}

pub fn load_language_config<'config, L: Language<'config>>(
    config_file_entry: &'config toml::Table,
    cli_matches: &'config clap::ArgMatches,
    args: &ArgsSet,
) -> anyhow::Result<L::Config> {

}

// #[cfg(test)]
// mod test {
//     use super::*;

//     const CURRENT_DIR: &str = env!("CARGO_MANIFEST_DIR");
//     const TEST_DIR: &str = "data/tests";

//     fn config_file_path(filename: &str) -> PathBuf {
//         [CURRENT_DIR, TEST_DIR, filename].iter().collect()
//     }

//     #[test]
//     fn default_test() {
//         let path = config_file_path("default_config.toml");
//         let config = load_config(Some(path)).unwrap();

//         assert_eq!(config, Config::default());
//     }

//     #[test]
//     fn empty_test() {
//         let path = config_file_path("empty_config.toml");
//         let config = load_config(Some(path)).unwrap();

//         assert_eq!(config, Config::default());
//     }

//     #[test]
//     fn mappings_test() {
//         let path = config_file_path("mappings_config.toml");
//         let config = load_config(Some(path)).unwrap();

//         assert_eq!(config.swift.type_mappings["DateTime"], "Date");
//         assert_eq!(config.kotlin.type_mappings["DateTime"], "String");
//         assert_eq!(config.scala.type_mappings["DateTime"], "String");
//         assert_eq!(config.typescript.type_mappings["DateTime"], "string");
//         #[cfg(feature = "go")]
//         assert_eq!(config.go.type_mappings["DateTime"], "string");
//     }

//     #[test]
//     fn decorators_test() {
//         let path = config_file_path("decorators_config.toml");
//         let config = load_config(Some(path)).unwrap();

//         assert_eq!(config.swift.default_decorators.len(), 1);
//         assert_eq!(config.swift.default_decorators[0], "Sendable");
//     }

//     #[test]
//     fn constraints_test() {
//         let path = config_file_path("constraints_config.toml");
//         let config = load_config(Some(path)).unwrap();

//         assert_eq!(config.swift.default_generic_constraints.len(), 1);
//         assert_eq!(config.swift.default_generic_constraints[0], "Sendable");
//     }

//     #[test]
//     fn swift_prefix_test() {
//         let path = config_file_path("swift_prefix_config.toml");
//         let config = load_config(Some(path)).unwrap();

//         assert_eq!(config.swift.prefix, "test");
//     }
// }
