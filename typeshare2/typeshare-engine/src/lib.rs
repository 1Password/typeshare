// mod config;
pub mod args;
pub mod config;
pub mod parser;
mod rename;
mod serde;
mod topsort;
mod type_parser;
mod visitors;
pub mod writer;

use std::{
    fmt::{self, Display, Write},
    io,
    path::PathBuf,
};

use indent_write::fmt::IndentWriter;
use proc_macro2::LineColumn;
use syn::spanned::Spanned;
use thiserror::Error;
use typeshare_model::prelude::{CrateName, TypeName};

// Re-export this for the driver crate to use
pub use typeshare_model::language::FilesMode;

#[derive(Debug, Error)]
pub struct FileParseErrors {
    pub path: PathBuf,
    pub crate_name: CrateName,
    pub kind: FileErrorKind,
}

impl FileParseErrors {
    pub fn new(path: PathBuf, crate_name: CrateName, kind: FileErrorKind) -> Self {
        Self {
            path,
            crate_name,
            kind,
        }
    }
}

impl Display for FileParseErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "in {path}, {error}",
            path = self.path.display(),
            error = self.kind
        )
    }
}

#[derive(Debug)]
pub enum FileErrorKind {
    ParseErrors(ParseErrorSet),
    ReadError(io::Error),
}

impl Display for FileErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileErrorKind::ParseErrors(parse_error_set) => parse_error_set.fmt(f),
            FileErrorKind::ReadError(error) => write!(f, "i/o error: {error}"),
        }
    }
}
/// A group of parse errors from a single file. Guaranteed to be non-emtpy.
#[derive(Debug)]
pub struct ParseErrorSet {
    errors: Vec<ParseError>,
}

impl ParseErrorSet {
    pub fn collect(errors: impl IntoIterator<Item = ParseError>) -> Result<(), Self> {
        let mut errors = errors.into_iter().peekable();

        match errors.peek() {
            Some(_) => Err(Self {
                errors: errors.collect(),
            }),
            None => Ok(()),
        }
    }
}

impl From<ParseError> for ParseErrorSet {
    fn from(error: ParseError) -> Self {
        Self {
            errors: Vec::from([error]),
        }
    }
}

impl Display for ParseErrorSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.errors.as_slice() {
            [] => Ok(()),
            [error] => write!(f, "{error}"),
            errors => {
                writeln!(f, "multiple errors:")?;
                let mut f = IndentWriter::new("  ", f);
                errors.iter().try_for_each(|error| write!(f, "{error}"))
            }
        }
    }
}

#[derive(Debug, Error)]
#[error("at {}:{}..{}:{}: {kind}",
    .start.line,
    .start.column,
    .end.line,
    .end.column,
)]
pub struct ParseError {
    start: LineColumn,
    end: LineColumn,
    kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(span: &impl Spanned, kind: ParseErrorKind) -> Self {
        let span = span.span();
        Self {
            start: span.start(),
            end: span.end(),
            kind,
        }
    }
}

/// Errors that can occur while parsing Rust source input.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ParseErrorKind {
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
