//! Error types for parsing.
use itertools::Itertools as _;
use proc_macro2::Span;
use thiserror::Error;

#[derive(Debug)]
/// Wrapper for a parse error which includes a span.
pub struct ParseErrorWithSpan {
    /// Parse error
    error: ParseError,
    /// Span
    span: Span,
}

impl std::error::Error for ParseErrorWithSpan {}

impl std::fmt::Display for ParseErrorWithSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, on line {} and column {}",
            self.error,
            self.span.start().line,
            self.span.start().column
        )
    }
}

/// Errors that can occur while parsing Rust source input.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ParseError {
    #[error("{0}")]
    SynError(#[from] syn::Error),
    #[error("Failed to parse a Rust type: {0}")]
    RustTypeParseError(#[from] RustTypeParseError),
    #[error("Unsupported language encountered: {0}")]
    UnsupportedLanguage(String),
    #[error("Unsupported type encountered: {0}")]
    UnsupportedType(String),
    #[error("Tuple structs with more than one field are currently unsupported")]
    ComplexTupleStruct,
    #[error("Multiple unnamed associated types are not currently supported")]
    MultipleUnnamedAssociatedTypes,
    #[error("The serde tag attribute is not supported for non-algebraic enums: {enum_ident}")]
    SerdeTagNotAllowed { enum_ident: String },
    #[error("The serde content attribute is not supported for non-algebraic enums: {enum_ident}")]
    SerdeContentNotAllowed { enum_ident: String },
    #[error("Serde tag attribute needs to be specified for algebraic enum {enum_ident}. e.g. #[serde(tag = \"type\", content = \"content\")]")]
    SerdeTagRequired { enum_ident: String },
    #[error("Serde content attribute needs to be specified for algebraic enum {enum_ident}. e.g. #[serde(tag = \"type\", content = \"content\")]")]
    SerdeContentRequired { enum_ident: String },
    #[error("The expression assigned to this constant variable is not a numeric literal")]
    RustConstExprInvalid,
    #[error("You cannot use typeshare on a constant that is not a numeric literal")]
    RustConstTypeInvalid,
    #[error("The serde flatten attribute is not currently supported")]
    SerdeFlattenNotAllowed,
    #[error("IO error: {0}")]
    IOError(String),
}

/// Parse error types that can capture a span and convert
/// into the top level [ParseErrorWithSpan] type.
pub trait WithSpan {
    /// Convert [Self] into a [`ParserErrorWithSpan`].
    fn with_span(self, span: Span) -> ParseErrorWithSpan;
}

impl WithSpan for RustTypeParseError {
    fn with_span(self, span: Span) -> ParseErrorWithSpan {
        ParseError::RustTypeParseError(self).with_span(span)
    }
}

impl WithSpan for ParseError {
    fn with_span(self, span: Span) -> ParseErrorWithSpan {
        ParseErrorWithSpan { error: self, span }
    }
}

#[derive(Debug, Error)]
/// Errors during file generation.
pub enum GenerationError {
    /// The post generation step failed.
    #[error("Post generation failed: {0}")]
    PostGeneration(String),
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum RustTypeParseError {
    #[error("Unsupported type: \"{}\"", .0.iter().join(","))]
    UnsupportedType(Vec<String>),
    #[error("Unexpected token when parsing type: `{0}`. This is an internal error, please ping a typeshare developer to resolve this problem.")]
    UnexpectedToken(String),
    #[error("Tuples are not allowed in typeshare types")]
    UnexpectedParameterizedTuple,
    #[error("Could not parse numeric literal")]
    NumericLiteral(syn::parse::Error),
}
