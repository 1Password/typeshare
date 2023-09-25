pub mod config;
pub mod type_mapping;

use std::fs::{create_dir_all, OpenOptions};
use std::path::PathBuf;
use std::{error::Error, fmt::Debug, io, io::Write, ops::Deref};

pub use config::LanguageConfig;
use itertools::Itertools;
use log::{error, info};
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use thiserror::Error;
pub use type_mapping::{TypeMapping, TypeMappingValue};

use crate::parsed_types::{
    AnonymousStructVariant, Comment, CommentLocation, DecoratorsMap, EnumVariant, Generics, Id,
    ParsedData, ParsedEnum, ParsedStruct, SpecialType, StructShared, Type, TypeError,
};

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum LanguageError<E: Error> {
    #[error("a type generation error occurred: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Language Error: {0}")]
    LanguageError(E),
    #[error("Type unable to be built: {0}")]
    UTF8Error(#[from] std::string::FromUtf8Error),
    #[error("Type unable to be parsed: {0}")]
    TypeParseError(#[from] TypeError),
    #[error("Formatting error: {0}")]
    FormattingError(#[from] std::fmt::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiFileItem {
    pub name: String,
    pub internal_type: String,
    pub content: String,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumIs)]
pub enum WriteTypesResult {
    MultiFile { files: Vec<MultiFileItem> },
    SingleFile(String),
}
impl WriteTypesResult {
    pub fn write_parse_with_default_file(self, out: PathBuf, default_file: &str) -> io::Result<()> {
        let out = if self.is_single_file() && out.exists() && out.is_dir() {
            out.join(default_file)
        } else {
            out
        };
        self.write_parse(out)
    }
    pub fn write_parse(self, out: PathBuf) -> io::Result<()> {
        match self {
            WriteTypesResult::MultiFile { files } => {
                if out.exists() {
                    if !out.is_dir() {
                        return Err(io::Error::new(
                            io::ErrorKind::AlreadyExists,
                            format!("{} is not a directory", out.display()),
                        ));
                    }
                } else {
                    create_dir_all(&out)?;
                }
                for multi_file_item in files {
                    let write_to = out.join(multi_file_item.name);
                    info!(
                        "Writing {} to {}",
                        multi_file_item.internal_type,
                        write_to.display()
                    );
                    let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(write_to)?;
                    if let Err(error) = file.write_all(multi_file_item.content.as_bytes()) {
                        error!("Error writing {}: {}", multi_file_item.internal_type, error);
                    }
                }
            }
            WriteTypesResult::SingleFile(single) => {
                if out.exists() && out.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!("{} is a directory", out.display()),
                    ));
                }
                let mut out = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(out)?;
                out.write_all(single.as_bytes())?;
                return Ok(());
            }
        }
        Ok(())
    }
}
pub type LangResult<T, E> = Result<T, LanguageError<E>>;
/// Language-specific state and processing.
///
/// The `Language` implementation is allowed to maintain mutable state, and it
/// is allowed to assume that a unique `Language` instance will be constructed
/// for each `Generator` instance.
pub trait Language {
    type Config: LanguageConfig;
    type Error: Error + Send + Sync + 'static;
    /// Should be lower camel case, e.g. `java`, `rust`, 'typescript' etc. a-z only.
    ///
    /// This is what is used to identify the language in the config file and inside the typeshare lang decorators
    fn language_name() -> &'static str
    where
        Self: Sized;
    fn extension(&self) -> &'static str;
    /// If more than one type is provided in the input this language will generate multiple files
    fn requires_multiple_files(&self) -> bool {
        false
    }

    fn get_config(&self) -> &Self::Config;

    /// Given `data`, generate type-code for this language and return it in a WriteTypeResult
    fn generate_from_parse(
        &mut self,
        data: &ParsedData,
    ) -> LangResult<WriteTypesResult, Self::Error>;
}

pub trait TypeFormatter: Language {
    /// Convert a Rust type into a type from this language.
    fn format_type(
        &mut self,
        ty: &Type,
        generic_types: &[String],
    ) -> LangResult<String, Self::Error> {
        match ty {
            Type::Simple { id } => self.format_simple_type(id, generic_types),
            Type::Generic { id, parameters } => {
                self.format_generic_type(id, parameters.as_slice(), generic_types)
            }
            Type::Special(special) => self.format_special_type(special, generic_types),
        }
    }

    // We need to pass in an &String for type mapping
    /// Format a simple type with no generic parameters.
    /// Note that we still need to take a list of generic types in case the implementors
    /// need to differentiate between a user-defined type and a generic type (for example: Swift)
    #[allow(clippy::ptr_arg)]
    fn format_simple_type(
        &mut self,
        base: &String,
        _generic_types: &[String],
    ) -> LangResult<String, Self::Error> {
        Ok(
            if let Some(mapped) = self.get_config().type_mappings().get(base) {
                mapped.to_string()
            } else {
                base.into()
            },
        )
    }

    // We need to pass in an &String for type mapping
    /// Format a generic type that takes in generic arguments, which
    /// may be recursive.
    #[allow(clippy::ptr_arg)]
    fn format_generic_type(
        &mut self,
        base: &String,
        parameters: &[Type],
        generic_types: &[String],
    ) -> LangResult<String, Self::Error> {
        if let Some(mapped) = self.get_config().type_mappings().get(base) {
            Ok(mapped.to_string())
        } else {
            let parameters: LangResult<Vec<String>, Self::Error> = parameters
                .iter()
                .map(|p| self.format_type(p, generic_types))
                .collect();
            let parameters = parameters?;
            Ok(format!(
                "{}{}",
                self.format_simple_type(base, generic_types)?,
                (!parameters.is_empty())
                    .then(|| self.format_generic_parameters(parameters))
                    .unwrap_or_default()
            ))
        }
    }

    /// Format generic parameters into A<T,R> which is common for many supported languages.
    /// Reimplement if other notations is used.
    fn format_generic_parameters(&mut self, parameters: Vec<String>) -> String {
        format!("<{}>", parameters.into_iter().join(", "))
    }

    /// Format a base type that is classified as a SpecialRustType.
    fn format_special_type(
        &mut self,
        special_ty: &SpecialType,
        generic_types: &[String],
    ) -> LangResult<String, Self::Error>;
}

/// Implement this this on types such as Enum or Structs that can be generated by your language
pub trait Generate<L: Language> {
    /// Generates to a String
    fn generate(&self, language: &mut L) -> Result<String, LanguageError<L::Error>> {
        let mut buffer = Vec::new();
        self.generate_to(language, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
    fn generate_to(
        &self,
        language: &mut L,
        write: &mut impl Write,
    ) -> Result<(), LanguageError<L::Error>>;
}

/// Write out named types to represent anonymous struct enum variants.
///
/// Take the following enum as an example:
///
/// ```
/// enum AlgebraicEnum {
///     AnonymousStruct { field: String, another_field: bool },
/// }
/// ```
///
/// This function will write out:
///
/// ```compile_fail
/// /// Generated type representing the anonymous struct variant `<make_struct_name>` of the `AlgebraicEnum` rust enum
/// /* the struct definition for whatever language */
/// ```
///
/// It does this by calling `write_struct` on the given `language_impl`. The
/// name of the struct is controlled by the `make_struct_name` closure; you're
/// given the variant name and asked to return whatever struct name works best
/// for your language.
pub fn covert_anonymous_structs_to_structs<F>(
    e: &ParsedEnum,
    make_struct_name: F,
) -> Vec<ParsedStruct>
where
    F: Fn(&str) -> String,
{
    let mut structs = Vec::new();
    for variant in &e.shared().variants {
        let EnumVariant::AnonymousStruct(AnonymousStructVariant { fields, shared }) = variant
        else {
            continue;
        };
        let struct_name = make_struct_name(&shared.id.original);

        // Builds the list of generic types (e.g [T, U, V]), by digging
        // through the fields recursively and comparing against the
        // enclosing enum's list of generic parameters.
        let generic_types: Vec<String> = fields
            .iter()
            .flat_map(|field| {
                e.deref()
                    .generic_types
                    .iter()
                    .filter(|g| field.ty.contains_type(g))
            })
            .unique()
            .cloned()
            .collect();
        let source = e.deref().source.push(&struct_name);

        let parsed_struct =ParsedStruct::TraditionalStruct {
                fields: fields.clone(),
                shared: StructShared {
                    source,
                    id: Id {
                        original: struct_name.clone(),
                        renamed: struct_name.clone(),
                        rename_all: None,
                    },
                    generic_types: Generics::from(generic_types),
                    comments: Comment::new_single(
                        format!(
                            "Generated type representing the anonymous struct variant `{}` of the `{}` Rust enum",
                            &shared.id.original,
                            &e.deref().id.original,
                        ),
                        CommentLocation::Type,
                    ),
                    decorators: DecoratorsMap::default(),
                },
            };
        structs.push(parsed_struct.clone());
    }
    structs
}
