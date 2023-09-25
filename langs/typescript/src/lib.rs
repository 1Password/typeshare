pub mod config;
mod lang_impl;

pub use lang_impl::TypeScript;
use typeshare_module::build_typeshare_module;
#[test]
pub fn test() {}
#[build_typeshare_module]
pub mod typeshare_module_impl {
    use crate::config::TypeScriptConfig;
    use crate::lang_impl::TypescriptError;
    use crate::TypeScript;
    use typeshare_core::language::{Language, LanguageError, WriteTypesResult};
    use typeshare_core::parsed_types::ParsedData;

    pub type TypeConfig = TypeScriptConfig;
    pub type LanguageType = TypeScript;
    pub static LANGUAGE_NAME: &str = "typescript";

    pub fn build_types(
        config: TypeScriptConfig,
        parsed_data: ParsedData,
    ) -> Result<(WriteTypesResult, String), LanguageError<TypescriptError>> {
        let mut typescript = TypeScript { config };
        typescript
            .generate_from_parse(&parsed_data)
            .map(|v| (v, typescript.config.default_file_name))
    }
    pub fn get_default_config() -> String {
        include_str!("./default_config.toml").to_string()
    }
}
