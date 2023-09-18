use clap::builder::Str;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};
use typeshare_core::language::TypeScriptEnumWriteMethod;
use typeshare_core::type_mapping::TypeMapping;

pub(crate) const DEFAULT_CONFIG_FILE_NAME: &str = "typeshare.toml";

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq, Args)]
#[serde(default)]
pub struct KotlinParams {
    #[clap(long = "java-package")]
    #[serde(rename = "package")]
    pub java_package: Option<String>,
    #[clap(long = "module-name")]
    #[serde(rename = "module_name")]
    pub kotlin_module_name: Option<String>,
    #[clap(skip)]
    pub type_mappings: TypeMapping,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq, Args)]
#[serde(default)]
pub struct ScalaParams {
    #[clap(long = "scala-package")]
    #[serde(rename = "package")]
    pub scala_package: Option<String>,
    #[clap(long = "scala-module-name")]
    #[serde(rename = "module_name")]
    pub scala_module_name: Option<String>,
    #[clap(skip)]
    pub type_mappings: TypeMapping,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq, Args)]
#[serde(default)]
pub struct SwiftParams {
    #[clap(long = "swift-prefix")]
    pub prefix: Option<String>,
    #[clap(skip)]
    pub type_mappings: TypeMapping,
    #[clap(skip)]
    pub default_decorators: Vec<String>,
    #[clap(skip)]
    pub default_generic_constraints: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(default)]
pub struct TypeScriptParams {
    pub enum_write_method: TypeScriptEnumWriteMethod,
    pub type_mappings: TypeMapping,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Args)]
#[serde(default)]
#[cfg(feature = "go")]
pub struct GoParams {
    #[clap(long = "go-package")]
    #[serde(rename = "package")]
    pub go_package: Option<String>,
    #[clap(skip)]
    pub type_mappings: TypeMapping,
    #[clap(skip)]
    pub uppercase_acronyms: Vec<String>,
}

/// The paramters that are used to configure the behaviour of typeshare
/// from the configuration file `typeshare.toml`
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(default)]
pub(crate) struct Config {
    #[serde(skip_serializing_if = "VecDeque::is_empty")]
    pub directories: VecDeque<String>,
    pub swift: SwiftParams,
    pub typescript: TypeScriptParams,
    pub kotlin: KotlinParams,
    pub scala: ScalaParams,
    #[cfg(feature = "go")]
    pub go: GoParams,
}

pub(crate) fn store_config(config: &Config, file_path: Option<&str>) -> Result<(), io::Error> {
    let file_path = file_path.unwrap_or(DEFAULT_CONFIG_FILE_NAME);

    // Fail if trying to overwrite an existing config file
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)?;

    let config_output =
        toml::to_string_pretty(config).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    file.write_all(config_output.as_bytes())?;

    Ok(())
}

pub(crate) fn load_config(file_path: impl Into<PathBuf>) -> Result<Config, io::Error> {
    let file_path = file_path.into();

    if file_path.exists() {
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

#[cfg(test)]
mod test {
    use super::*;

    const CURRENT_DIR: &str = env!("CARGO_MANIFEST_DIR");
    const TEST_DIR: &str = "data/tests";

    fn config_file_path(filename: &str) -> PathBuf {
        [CURRENT_DIR, TEST_DIR, filename].iter().collect()
    }

    #[test]
    fn default_test() {
        let path = config_file_path("default_config.toml");
        let config = load_config(Some(path)).unwrap();

        assert_eq!(config, Config::default());
    }

    #[test]
    fn empty_test() {
        let path = config_file_path("empty_config.toml");
        let config = load_config(Some(path)).unwrap();

        assert_eq!(config, Config::default());
    }

    #[test]
    fn mappings_test() {
        let path = config_file_path("mappings_config.toml");
        let config = load_config(Some(path)).unwrap();

        assert_eq!(
            config.swift.type_mappings["DateTime"],
            "Date".parse().unwrap()
        );
        assert_eq!(
            config.kotlin.type_mappings["DateTime"],
            "String".parse().unwrap()
        );
        assert_eq!(
            config.scala.type_mappings["DateTime"],
            "String".parse().unwrap()
        );
        assert_eq!(
            config.typescript.type_mappings["DateTime"],
            "string".parse().unwrap()
        );
        #[cfg(feature = "go")]
        assert_eq!(config.go.type_mappings["DateTime"], "string");
    }

    #[test]
    fn decorators_test() {
        let path = config_file_path("decorators_config.toml");
        let config = load_config(Some(path)).unwrap();

        assert_eq!(config.swift.default_decorators.len(), 1);
        assert_eq!(config.swift.default_decorators[0], "Sendable");
    }

    #[test]
    fn constraints_test() {
        let path = config_file_path("constraints_config.toml");
        let config = load_config(Some(path)).unwrap();

        assert_eq!(config.swift.default_generic_constraints.len(), 1);
        assert_eq!(config.swift.default_generic_constraints[0], "Sendable");
    }

    #[test]
    fn swift_prefix_test() {
        let path = config_file_path("swift_prefix_config.toml");
        let config = load_config(Some(path)).unwrap();

        assert_eq!(config.swift.prefix, "test");
    }
}
