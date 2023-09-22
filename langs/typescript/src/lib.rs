pub mod config;
mod lang_impl;

use crate::config::TypeScriptConfig;
pub use lang_impl::TypeScript;
use std::str;

#[cfg(feature = "cli")]
use typeshare_core::define_command;

use typeshare_core::FFILanguageDescription;
pub const DESCRIPTION: FFILanguageDescription = FFILanguageDescription::new(
    "typescript",
    env!("CARGO_PKG_VERSION"),
    env!("VERGEN_RUSTC_SEMVER"),
    #[cfg(feature = "cli")]
    true,
    #[cfg(not(feature = "cli"))]
    false,
);

#[cfg(feature = "cli")]
define_command!(TypeScriptConfig, TypeScript);
#[cfg(feature = "cli")]
pub mod ffi {
    use super::{execute_inner, GenerateCommand, DESCRIPTION};
    use crate::config::{DEFAULT_CONFIG};
    use log::error;
    use std::ffi::{c_char, CString};
    
    use std::mem;
    
    use std::ptr::null;
    use typeshare_core::cli::clap::Parser;
    use typeshare_core::cli::ffi::to_args_array;
    
    use typeshare_core::FFILanguageDescription;

    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn description() -> FFILanguageDescription {
        DESCRIPTION
    }

    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn generate_default_config() -> *const c_char {
        let Ok(string) = CString::new(DEFAULT_CONFIG) else {
            error!("Could not Load Default Config");
            return null();
        };
        let ptr = string.as_ptr();
        mem::forget(string);
        ptr
    }
    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn execute(args: *const *const c_char, size: usize) {
        match to_args_array(args, size) {
            Ok(ok) => {
                let command = GenerateCommand::parse_from(ok);
                execute_inner(command);
            }
            Err(err) => {
                eprintln!("Could not Parse Arguments: {}", err);
                std::process::exit(1);
            }
        }
    }
}

#[doc(hidden)]
#[cfg(feature = "cli")]
pub fn execute_inner(generate_command: GenerateCommand) {
    let (config, lang_config, output) = generate_command.load_or_exit();
    let mut lang = TypeScript {
        config: lang_config,
    };

    let Err(error) =
        typeshare_core::process_directories_and_write(&mut lang, config.directories, output)
    else {
        println!("Success!");
        std::process::exit(0);
    };

    eprintln!("Error: {}", error);
    std::process::exit(1);
}
