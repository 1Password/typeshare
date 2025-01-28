// mod config;
mod parser;
mod rename;
mod type_parser;
mod visitors;
mod writer;

use syn::token::Type;
use thiserror::Error;
use typeshare_model::prelude::{CrateName, TypeName};

pub use parser::{parse_input, parser_inputs, LangConfig, ParserInput};
pub use writer::{write_generated, OutputMode};

/// Errors that can occur while parsing Rust source input.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ParseError {
    #[error("{0}")]
    SynError(#[from] syn::Error),
    #[error("failed to parse a rust type: {0}")]
    RustTypeParseError(#[from] RustTypeParseError),
    #[error("unsupported language encountered: {0}")]
    UnsupportedLanguage(String),
    #[error("unsupported type encountered: {0}")]
    UnsupportedType(String),
    #[error("tuple structs with more than one field are currently unsupported")]
    ComplexTupleStruct,
    #[error("multiple unnamed associated types are not currently supported")]
    MultipleUnnamedAssociatedTypes,
    #[error("the serde tag attribute is not supported for non-algebraic enums: {enum_ident}")]
    SerdeTagNotAllowed { enum_ident: TypeName },
    #[error("the serde content attribute is not supported for non-algebraic enums: {enum_ident}")]
    SerdeContentNotAllowed { enum_ident: TypeName },
    #[error("serde tag attribute needs to be specified for algebraic enum {enum_ident}. e.g. #[serde(tag = \"type\", content = \"content\")]")]
    SerdeTagRequired { enum_ident: TypeName },
    #[error("serde content attribute needs to be specified for algebraic enum {enum_ident}. e.g. #[serde(tag = \"type\", content = \"content\")]")]
    SerdeContentRequired { enum_ident: TypeName },
    #[error("the serde flatten attribute is not currently supported")]
    SerdeFlattenNotAllowed,
}

/// Error with it's related data.
#[derive(Debug)]
pub struct ErrorInfo {
    /// The crate where this error occured.
    pub crate_name: CrateName,
    /// The file name being parsed.
    pub file_name: String,
    /// The parse error.
    pub error: ParseError,
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum RustTypeParseError {
    #[error("{0:?}")]
    UnsupportedType(Vec<String>),
    #[error("Unexpected token when parsing type: `{0}`. This is an internal error, please ping a typeshare developer to resolve this problem.")]
    UnexpectedToken(String),
    #[error("Tuples are not allowed in typeshare types")]
    UnexpectedParameterizedTuple,
    #[error("Could not parse numeric literal")]
    NumericLiteral(syn::parse::Error),
}
