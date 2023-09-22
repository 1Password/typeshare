use std::{
    collections::{HashMap, VecDeque},
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::language::TypeMapping;

pub(crate) const DEFAULT_CONFIG_FILE_NAME: &str = "typeshare.toml";
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct CommonConfig {
    /// Any Value inside the Type Mapping will be assumed to be a Rust Type
    pub type_mappings: TypeMapping,
}

/// The parameters that are used to configure the behaviour of typeshare
/// from the configuration file `typeshare.toml`
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(default)]
pub struct Config {
    #[serde(skip_serializing_if = "VecDeque::is_empty")]
    pub directories: VecDeque<String>,
    pub common: CommonConfig,
    pub language: HashMap<String, Value>,
}
impl Config {
    pub fn provided_directories(&mut self, directories: Vec<String>) {
        if !directories.is_empty() {
            self.directories = VecDeque::from(directories);
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Config {
            directories: VecDeque::from_iter(vec![".".to_string()]),
            common: CommonConfig::default(),
            language: HashMap::new(),
        }
    }
}
pub fn store_config<P: AsRef<Path>>(
    config: &Config,
    file_path: Option<&P>,
) -> Result<(), io::Error> {
    let file_path = file_path
        .map(|v| v.as_ref().to_path_buf())
        .unwrap_or(PathBuf::from(DEFAULT_CONFIG_FILE_NAME));

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

pub fn load_config(file_path: impl Into<PathBuf>) -> Result<Config, io::Error> {
    let file_path = file_path.into();

    if file_path.exists() {
        let config_string = fs::read_to_string(file_path)?;
        toml::from_str(&config_string).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    } else {
        Ok(Config::default())
    }
}

/// Search each ancestor directory for configuration file
pub(crate) fn find_configuration_file() -> Option<PathBuf> {
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
mod tests {
    #[test]
    pub fn load_example() {
        let config = super::load_config("./typeshare.example.toml");
        println!("{:?}", config);
    }
}
