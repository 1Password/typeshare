#![deny(improper_ctypes_definitions)]

use crate::language_logger::{LanguageLogger, LanguageLoggerConfig};
pub use toml;

pub mod argument;
pub mod ffi_interop;
pub mod language_logger;
pub mod language_module;

pub use bincode;
pub use log;

pub fn init_logger(crate_name: &'static str, log_config: LanguageLoggerConfig) {
    let language_logger = LanguageLogger {
        language_crate: crate_name,
        language_crate_level: log_config.language_crate_level,
        typeshare_core_log_level: log_config.typeshare_core_level,
    };
    let _ = log::set_boxed_logger(Box::new(language_logger));
}
/// Initializes the logger used by typeshare
pub fn debug_logger() {
    simple_log::quick!();
}
#[cfg(feature = "macros")]
pub use typeshare_lang_module_macros::build_typeshare_module;

#[cfg(test)]
mod tests {
    #[test]
    pub fn test() {}
}
