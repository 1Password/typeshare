use crate::ffi::LanguageLibrary;
use crate::Error;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use std::fmt::Display;
use std::fs::File;

use std::io::Write;
use std::path::PathBuf;
use tabled::Table;
use toml::Value;
use typeshare_core::LanguageDescription;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageEntry {
    pub language: LanguageDescription,
    pub lib_path: PathBuf,
    pub default_config: Option<String>, // TODO: Add Default Config and CLI Layout for Auto Completion
}
impl Display for LanguageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.language.name(), self.lib_path.display())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CLIConfig {
    pub languages_dir: PathBuf,
    pub languages: Vec<LanguageEntry>,
}

impl CLIConfig {
    pub fn print_info(&self) {
        let typeshare_home = crate::config::get_typeshare_directory();
        println!("Typeshare home: {}", typeshare_home.display());
        println!("Languages directory: {}", self.languages_dir.display());
        let table = Table::new(self.languages.iter().map(|v| &v.language));
        println!("{}", table);
    }
}
impl CLIConfig {
    pub fn get_app_config() -> Result<Self, Error> {
        let typeshare_home = get_typeshare_directory();
        if !typeshare_home.exists() {
            std::fs::create_dir_all(&typeshare_home)?;
        }
        let config_path = typeshare_home.join("config.toml");
        debug!("Config path: {}", config_path.display());
        return if !config_path.exists() {
            let languages_home = typeshare_home.join("languages");
            if !languages_home.exists() {
                std::fs::create_dir_all(&languages_home)?;
            }
            let mut file = File::create(&config_path)?;
            let mut config = CLIConfig {
                languages_dir: languages_home,
                languages: vec![],
            };
            config.rebuild_languages()?;
            let toml = toml::to_string_pretty(&config)?;
            file.write_all(toml.as_bytes())?;

            info!(
                "First Run Detected, Created config file at {}",
                config_path.display()
            );
            config.print_info();
            Ok(config)
        } else {
            let config = toml::from_str(&std::fs::read_to_string(&config_path)?)?;
            Ok(config)
        };
    }
    pub fn save(&self) -> Result<(), Error> {
        let typeshare_home = get_typeshare_directory();
        if !typeshare_home.exists() {
            std::fs::create_dir_all(&typeshare_home)?;
        }
        let config_path = typeshare_home.join("config.toml");
        debug!("Config path: {}", config_path.display());
        let mut file = File::create(&config_path)?;
        let toml = toml::to_string_pretty(&self)?;
        file.write_all(toml.as_bytes())?;
        Ok(())
    }
    /// Rebuilds the Languages from the languages directory.
    ///
    /// Rebuilding is required after adding a new language to the languages directory or updating an existing language.
    pub fn rebuild_languages(&mut self) -> Result<(), Error> {
        let mut languages = vec![];
        if !self.languages_dir.exists() {
            std::fs::create_dir_all(&self.languages_dir)?;
            info!("Created languages directory");
            return Ok(());
        }
        for entries in self.languages_dir.read_dir()? {
            let entry = entries?;
            let path = entry.path();
            if !LanguageLibrary::can_load(&path) {
                debug!("Skipping {:?}", path);
                continue;
            }
            let library = unsafe { LanguageLibrary::load(path) };
            let library = match library {
                Ok(ok) => ok,
                Err(err) => {
                    warn!("Unable to load library {}: {}", entry.path().display(), err);
                    continue;
                }
            };
            let description = LanguageDescription::from(library.call_description()?);
            let default_config = library.get_default_config()?;
            if let Some(default_config) = &default_config {
                toml::from_str::<Value>(default_config)?;
            }
            let path = library.unload()?;
            let language_entry = LanguageEntry {
                language: description,
                lib_path: path,
                default_config,
                // Add CLI Layout for Auto Completion
            };
            debug!("Found language {}", language_entry);
            languages.push(language_entry);
        }
        if languages.is_empty() {
            warn!("No languages found in languages directory");
        }
        self.languages = languages;
        Ok(())
    }
    pub fn get_language(&self, name: &str) -> Option<&LanguageEntry> {
        self.languages.iter().find(|v| v.language.name() == name)
    }
}
impl Default for CLIConfig {
    fn default() -> Self {
        CLIConfig {
            languages_dir: get_typeshare_directory().join("languages"),
            languages: vec![],
        }
    }
}
pub fn initialize() -> Result<(), Error> {
    let typeshare_dir = get_typeshare_directory();
    if !typeshare_dir.exists() {
        std::fs::create_dir_all(&typeshare_dir)?;
    }
    let typeshare_config = typeshare_dir.join("config.toml");
    if !typeshare_config.exists() {
        let mut file = File::create(&typeshare_config)?;
        let config = CLIConfig::default();
        if !config.languages_dir.exists() {
            std::fs::create_dir_all(&config.languages_dir)?;
        }
        let toml = toml::to_string_pretty(&config)?;
        file.write_all(toml.as_bytes())?;
    }
    Ok(())
}
pub fn get_typeshare_directory() -> PathBuf {
    match std::env::var("TYPESHARE_HOME").ok() {
        None => {
            let option = dirs::home_dir();
            match option {
                None => {
                    warn!("TYPESHARE_HOME is not set and could not find home directory, using .typeshare in current directory");
                    PathBuf::from(".typeshare")
                }
                Some(ok) => ok.join(".typeshare"),
            }
        }
        Some(ok) => PathBuf::from(ok),
    }
}
