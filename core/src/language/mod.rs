mod comment;
pub mod config;
pub mod type_mapping;

use crate::{
    parser::ParsedData,
    rust_types::{Id, RustEnum, RustEnumVariant, RustItem, RustStruct, RustTypeAlias},
    topsort::topsort,
};
pub use comment::{Comment, CommentLocation};
pub use config::{CommonConfig, LanguageConfig};
use itertools::Itertools;
pub use type_mapping::{TypeMapping, TypeMappingValue};

use log::{debug, error};

use serde::{Deserialize, Serialize};
use std::error::Error;

use std::{collections::HashMap, fmt::Debug, io::Write};
use thiserror::Error;

use crate::rust_types::{
    Generics, RustType, RustTypeFormatError, RustTypeParseError, SpecialRustType,
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
    #[error(transparent)]
    TypeFormatError(#[from] RustTypeFormatError),
    #[error(transparent)]
    TypeParseError(#[from] RustTypeParseError),
    #[error("Formatting error: {0}")]
    FormattingError(#[from] std::fmt::Error),
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiFileItem {
    pub name: String,
    pub internal_type: String,
    pub content: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteTypesResult {
    MultiFile { files: Vec<MultiFileItem> },
    SingleFile(String),
}

pub type LangResult<T, E> = std::result::Result<T, LanguageError<E>>;
/// Language-specific state and processing.
///
/// The `Language` implementation is allowed to maintain mutable state, and it
/// is allowed to assume that a unique `Language` instance will be constructed
/// for each `Generator` instance.
pub trait Language {
    type Config: LanguageConfig;
    type Error: Error + Send + Sync + 'static;
    fn language_name(&self) -> &'static str;
    fn extension(&self) -> &'static str;
    fn multi_file(&self) -> bool {
        false
    }

    fn get_config(&self) -> &Self::Config;

    /// Given `data`, generate type-code for this language and write it out to `writable`.
    /// Returns whether or not writing was successful.
    fn generate_types(&mut self, data: &ParsedData) -> LangResult<WriteTypesResult, Self::Error> {
        if self.multi_file() {
            panic!("Function generate_types should be overriden for multi file languages");
        }
        let mut file: Vec<u8> = Vec::new();
        self.begin_file(&mut file)?;

        let mut items: Vec<RustItem> = vec![];

        for a in &data.aliases {
            items.push(RustItem::Alias(a.clone()))
        }

        for s in &data.structs {
            items.push(RustItem::Struct(s.clone()))
        }

        for e in &data.enums {
            items.push(RustItem::Enum(e.clone()))
        }

        let sorted = topsort(items.iter().collect());

        for &thing in &sorted {
            match thing {
                RustItem::Enum(e) => self.write_enum(&mut file, e)?,
                RustItem::Struct(s) => self.write_struct(&mut file, s)?,
                RustItem::Alias(a) => self.write_type_alias(&mut file, a)?,
            }
        }

        self.end_file(&mut file)?;

        Ok(WriteTypesResult::SingleFile(String::from_utf8(file)?))
    }
    /// Convert a Rust type into a type from this language.
    fn format_type(
        &mut self,
        ty: &RustType,
        generic_types: &[String],
    ) -> LangResult<String, Self::Error> {
        match ty {
            RustType::Simple { id } => self.format_simple_type(id, generic_types),
            RustType::Generic { id, parameters } => {
                self.format_generic_type(id, parameters.as_slice(), generic_types)
            }
            RustType::Special(special) => self.format_special_type(special, generic_types),
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
                mapped.to_string().into()
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
        parameters: &[RustType],
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
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> LangResult<String, Self::Error>;

    fn write_comment(
        &mut self,
        w: &mut impl Write,
        comment: &Comment<'_>,
    ) -> LangResult<(), Self::Error>;
    /// Implementors can use this function to write a header for typeshared code
    fn begin_file(&mut self, w: &mut impl Write) -> LangResult<(), Self::Error> {
        let config_header = self.get_config().file_header();
        if let Some(config_header) = config_header.map(|s| s.to_owned()) {
            debug!("Writing file header: {}", config_header);
            self.write_comment(
                w,
                &Comment::new_single(config_header, CommentLocation::FileHeader),
            )?;
        } else {
            debug!("No file header to write");
        }
        Ok(())
    }

    /// Implementors can use this function to write a footer for typeshared code
    fn end_file(&mut self, _w: &mut impl Write) -> LangResult<(), Self::Error> {
        Ok(())
    }

    /// Write a type alias by converting it.
    /// Example of a type alias:
    /// ```
    /// type MyTypeAlias = String;
    /// ```
    fn write_type_alias(
        &mut self,
        _w: &mut impl Write,
        _t: &RustTypeAlias,
    ) -> LangResult<(), Self::Error> {
        Ok(())
    }

    /// Write a struct by converting it
    /// Example of a struct:
    /// ```ignore
    /// #[typeshare]
    /// #[derive(Serialize, Deserialize)]
    /// struct Foo {
    ///     bar: String
    /// }
    /// ```
    fn write_struct(
        &mut self,
        _w: &mut impl Write,
        _rs: &RustStruct,
    ) -> LangResult<(), Self::Error> {
        Ok(())
    }

    /// Write an enum by converting it.
    /// Example of an enum:
    /// ```ignore
    /// #[typeshare]
    /// #[derive(Serialize, Deserialize)]
    /// #[serde(tag = "type", content = "content")]
    /// enum Foo {
    ///     Fizz,
    ///     Buzz { yep_this_works: bool }
    /// }
    /// ```
    fn write_enum(&mut self, _w: &mut impl Write, _e: &RustEnum) -> LangResult<(), Self::Error> {
        Ok(())
    }

    /// Write out named types to represent anonymous struct enum variants.
    ///
    /// Take the following enum as an example:
    ///
    /// ```
    /// enum AlgebraicEnum {
    ///     AnonymousStruct {
    ///         field: String,
    ///         another_field: bool,
    ///     },
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
    fn write_types_for_anonymous_structs<F>(
        &mut self,
        w: &mut impl Write,
        e: &RustEnum,
        make_struct_name: F,
    ) -> LangResult<(), Self::Error>
    where
        F: Fn(&str) -> String,
    {
        for (fields, shared) in e.shared().variants.iter().filter_map(|v| match v {
            RustEnumVariant::AnonymousStruct { fields, shared } => Some((fields, shared)),
            _ => None,
        }) {
            let struct_name = make_struct_name(&shared.id.original);

            // Builds the list of generic types (e.g [T, U, V]), by digging
            // through the fields recursively and comparing against the
            // enclosing enum's list of generic parameters.
            let generic_types: Vec<String> = fields
                .iter()
                .flat_map(|field| {
                    e.shared()
                        .generic_types
                        .iter()
                        .filter(|g| field.ty.contains_type(g))
                })
                .unique()
                .cloned()
                .collect();
            let source = e.shared().source.push(&struct_name);

            self.write_struct(
                w,
                &RustStruct {
                    source,
                    id: Id {
                        original: struct_name.clone(),
                        renamed: struct_name.clone(),
                    },
                    fields: fields.clone(),
                    generic_types: Generics::from(generic_types),
                    comments: Comment::new_single(
                        format!(
                            "Generated type representing the anonymous struct variant `{}` of the `{}` Rust enum",
                            &shared.id.original,
                            &e.shared().id.original,
                        ),
                        CommentLocation::Type,
                    ),
                    decorators: HashMap::new(),
                },
            )?;
        }
        Ok(())
    }
}
