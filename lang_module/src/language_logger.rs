use console::style;
use log::{Level, LevelFilter, Log, Metadata, Record};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(default)]
pub struct LanguageLoggerConfig {
    pub language_crate_level: LevelFilter,
    pub typeshare_core_level: LevelFilter,
}
impl Default for LanguageLoggerConfig {
    fn default() -> Self {
        Self {
            language_crate_level: LevelFilter::Info,
            typeshare_core_level: LevelFilter::Warn,
        }
    }
}
#[cfg(feature = "ffi_v1")]
mod ffi_v1 {
    use super::LanguageLoggerConfig;
    use crate::ffi_interop::ffi_v1::FFILanguageLoggerConfig;
    impl From<FFILanguageLoggerConfig> for LanguageLoggerConfig {
        fn from(value: FFILanguageLoggerConfig) -> Self {
            let FFILanguageLoggerConfig {
                language_crate_level,
                typeshare_core_level,
            } = value;
            Self {
                language_crate_level,
                typeshare_core_level,
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageLogger {
    pub language_crate: &'static str,
    pub language_crate_level: LevelFilter,
    pub typeshare_core_log_level: LevelFilter,
}

impl Log for LanguageLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let name = if record
            .module_path()
            .map(|v| v.starts_with(self.language_crate))
            .unwrap_or(false)
        {
            if record.level() < self.language_crate_level {
                return;
            }
            self.language_crate
        } else if record
            .module_path()
            .map(|v| v.starts_with("typeshare_core"))
            .unwrap_or(false)
        {
            if record.level() < self.typeshare_core_log_level {
                return;
            }
            "typeshare_core"
        } else {
            return;
        };

        if record.level() == Level::Info {
            println!("[{}]: {}", name, record.args());
            return;
        }
        let tag = if record.level() == Level::Error {
            style("[ERROR]").red()
        } else if record.level() == Level::Warn {
            style("[WARN]").yellow()
        } else {
            style("[DEBUG]")
        };
        println!("{} [{}]: {}", tag, name, record.args());
    }

    fn flush(&self) {
        // do nothing
    }
}
