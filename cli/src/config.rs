use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

const DEFAULT_CONFIG_FILE_NAME: &str = "typeshare.toml";

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct KotlinParams {
    pub package: String,
    pub module_name: String,
    pub type_mappings: HashMap<String, String>,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct ScalaParams {
    pub package: String,
    pub module_name: String,
    pub type_mappings: HashMap<String, String>,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct SwiftParams {
    pub prefix: String,
    pub type_mappings: HashMap<String, String>,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(default)]
pub struct TypeScriptParams {
    pub type_mappings: HashMap<String, String>,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(default)]
#[cfg(feature = "go")]
pub struct GoParams {
    pub package: String,
    pub type_mappings: HashMap<String, String>,
    pub uppercase_acronyms: Vec<String>,
}

/// The paramters that are used to configure the behaviour of typeshare
/// from the configuration file `typeshare.toml`
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
#[serde(default)]
pub(crate) struct Config {
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

pub(crate) fn load_config<P>(file_path: Option<P>) -> Result<Config, io::Error>
where
    PathBuf: From<P>,
{
    let file_path = file_path
        .map(PathBuf::from)
        .or_else(find_configuration_file);

    if let Some(file_path) = file_path {
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

        assert_eq!(config.swift.type_mappings["DateTime"], "Date");
        assert_eq!(config.kotlin.type_mappings["DateTime"], "String");
        assert_eq!(config.typescript.type_mappings["DateTime"], "string");
        assert_eq!(config.go.type_mappings["DateTime"], "string");
    }

    #[test]
    fn swift_prefix_test() {
        let path = config_file_path("swift_prefix_config.toml");
        let config = load_config(Some(path)).unwrap();

        assert_eq!(config.swift.prefix, "test");
    }
}
