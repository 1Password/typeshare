use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::{ser, Deserialize, Serialize};
use typeshare_model::Language;

use crate::serde::{args::ArgsSetSerializer, config::ConfigDeserializer, empty::EmptyDeserializer};

pub use crate::serde::args::CliArgsSet;

const DEFAULT_CONFIG_FILE_NAME: &str = "typeshare.toml";

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GlobalConfig {
    /// If present, only fields / variants / items that are accepted by at
    /// least one of these os's will be emitted.
    #[serde(default)]
    pub target_os: Option<Vec<String>>,
}

/// A partially parsed typeshare config file.
///
/// This contains a `toml::Table` for each language that was found in the config
/// file. The `Config` type on the `Language` trait can be further deserialized
/// from this toml table. It also contains config that's specific to typeshare
/// itself.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    /// toml::Table doesn't have a const constructor, so there's not an easy
    /// way to make a long-lived empty table to deserialize from when the
    /// language is absent from the raw data. So we just put one here.
    ///
    /// This should literally always be empty.
    #[serde(skip)]
    empty: toml::Table,

    /// When we load the typeshare config file, we don't know precisely which
    /// languages we're going to have yet. So we parse them into arbitrary
    /// toml, keyed by language, which we will later deserialize into a
    /// specific language's config type.
    #[serde(flatten)]
    raw_data: BTreeMap<String, toml::Table>,

    // General config for typeshare, separate from any particular language
    #[serde(default)]
    typeshare: GlobalConfig,
}

impl Config {
    /// Retrieve the config for the given language, by deserializing it into
    /// the given type. The deserialize implementation should be able to handle
    /// arbitrary missing keys by populating them with default values. Errors
    ///
    pub fn config_for_language(&self, language: &str) -> &toml::Table {
        self.raw_data.get(language).unwrap_or(&self.empty)
    }

    // Store a config for a language, overriding the existing one, by
    // serializing the config type into a toml table
    pub fn store_config_for_language<T: Serialize>(
        &mut self,
        _language: &str,
        _config: &T,
    ) -> anyhow::Result<()> {
        todo!()
        // self.raw_data.insert(
        //     language.to_owned(),
        //     config
        //         .serialize(TableSerializer)
        //         .context("error converting config to toml")?,
        // );

        // Ok(())
    }

    pub fn global_config(&self) -> &GlobalConfig {
        &self.typeshare
    }
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.raw_data.serialize(serializer)
    }
}

pub fn compute_args_set<'a, L: Language<'a>>() -> anyhow::Result<CliArgsSet> {
    let empty_config = L::Config::deserialize(EmptyDeserializer).context(
        "failed to create empty config; \
        did you forget `#[serde(default)]`?",
    )?;

    let args_set = empty_config
        .serialize(ArgsSetSerializer::new(L::NAME))
        .context("failed to compute CLI arguments from language configuration type")?;

    Ok(args_set)
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
        None => match find_configuration_file() {
            None => return Ok(Config::default()),
            Some(path) => {
                file_path_buf = path;
                &file_path_buf
            }
        },
    };

    let config_string = fs::read_to_string(file_path).with_context(|| {
        format!(
            "i/o error reading typeshare config from '{path}'",
            path = file_path.display()
        )
    })?;

    toml::from_str(&config_string).with_context(|| {
        format!(
            "error loading typeshare config from '{path}'",
            path = file_path.display()
        )
    })
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

pub fn load_language_config<'a, 'config, L: Language<'config>>(
    config_file_entry: &'config toml::Table,
    cli_matches: &'config clap::ArgMatches,
    spec: &'a CliArgsSet,
) -> anyhow::Result<L::Config> {
    L::Config::deserialize(ConfigDeserializer::new(
        config_file_entry,
        cli_matches,
        spec,
    ))
    .context("error deserializing config")
}

pub fn load_language_config_from_file_and_args<'a, 'config, L: Language<'config>>(
    config: &'config Config,
    cli_matches: &'config clap::ArgMatches,
    spec: &'a CliArgsSet,
) -> anyhow::Result<L::Config> {
    load_language_config::<L>(config.config_for_language(L::NAME), cli_matches, spec)
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
