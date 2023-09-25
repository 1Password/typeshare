mod decorator;
mod enum_parse;
/// Implements Parsing for some types found inside types module
mod parse_types;
mod serde_parse;
mod struct_parse;
mod typeshare_attrs;

use proc_macro2::{Ident, Span};
use serde::{Deserialize, Serialize};
use syn::{Attribute, Expr, ItemType, Meta};
use thiserror::Error;

use crate::parse_types::{ParseGenerics, ParseType};
use crate::{enum_parse::parse_enum, struct_parse::parse_struct, typeshare_attrs::TypeShareAttrs};
use typeshare_core::parser::Parser;
use typeshare_core::{
    parsed_types::{Comment, CommentLocation, Id, ParsedData, ParsedTypeAlias, Source, TypeError},
    rename::{RenameAll, RenameExt},
};

pub const TYPESHARE_ATTR: &str = "typeshare";

/// Errors that can occur while parsing Rust source input.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ParseError {
    #[error("{0}")]
    SynError(#[from] syn::Error),
    #[error("failed to parse a rust type: {0}")]
    TypeError(#[from] TypeError),
    #[error("unsupported language encountered: {0}")]
    UnsupportedLanguage(String),
    #[error("unsupported type encountered: {0}")]
    UnsupportedType(String),
    #[error("tuple structs with more than one field are currently unsupported")]
    ComplexTupleStruct,
    #[error("multiple unnamed associated types are not currently supported")]
    MultipleUnnamedAssociatedTypes,
    #[error("the serde tag attribute is not supported for non-algebraic enums: {enum_ident}")]
    SerdeTagNotAllowed { enum_ident: String },
    #[error("the serde content attribute is not supported for non-algebraic enums: {enum_ident}")]
    SerdeContentNotAllowed { enum_ident: String },
    #[error("serde tag attribute needs to be specified for algebraic enum {enum_ident}. e.g. #[serde(tag = \"type\", content = \"content\")]")]
    SerdeTagRequired { enum_ident: String },
    #[error("serde content attribute needs to be specified for algebraic enum {enum_ident}. e.g. #[serde(tag = \"type\", content = \"content\")]")]
    SerdeContentRequired { enum_ident: String },
    #[error("the serde flatten attribute is not currently supported")]
    SerdeFlattenNotAllowed,
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RustParserConfig {}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RustParser {
    pub config: RustParserConfig,
}
impl RustParser {
    pub fn new(config: RustParserConfig) -> Self {
        Self { config }
    }
}

impl Parser for RustParser {
    type Error = ParseError;
    type Config = RustParserConfig;

    fn file_type() -> &'static str
    where
        Self: Sized,
    {
        "rust"
    }

    fn parser_name() -> &'static str
    where
        Self: Sized,
    {
        "Rust Language Parser"
    }

    fn file_extensions() -> Vec<&'static str>
    where
        Self: Sized,
    {
        vec!["rs"]
    }

    fn parse_into_from_str<I: AsRef<str>>(
        &self,
        input: I,
        target: &mut ParsedData,
        source: Source,
    ) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        // We will only produce output for files that contain the `#[typeshare]`
        // attribute, so this is a quick and easy performance win
        if !input.as_ref().contains("typeshare") {
            return Ok(());
        }

        // Parse and process the input, ensuring we parse only items marked with
        // `#[typeshare]
        let syn_file = syn::parse_file(input.as_ref())?;

        for item in flatten_items(syn_file.items.iter()) {
            match item {
                syn::Item::Struct(s) if has_typeshare_annotation(&s.attrs) => {
                    target.push_item(parse_struct(s, source.clone())?);
                }
                syn::Item::Enum(e) if has_typeshare_annotation(&e.attrs) => {
                    target.push_item(parse_enum(e, source.clone())?);
                }
                syn::Item::Type(t) if has_typeshare_annotation(&t.attrs) => {
                    target.aliases.push(parse_type_alias(t, source.clone())?);
                }
                _ => {}
            }
        }

        Ok(())
    }
}

/// Given an iterator over items, will return an iterator that flattens the contents of embedded
/// module items into the iterator.
fn flatten_items<'a>(
    items: impl Iterator<Item = &'a syn::Item>,
) -> impl Iterator<Item = &'a syn::Item> {
    items.flat_map(|item| {
        match item {
            syn::Item::Mod(syn::ItemMod {
                content: Some((_, items)),
                ..
            }) => flatten_items(items.iter()).collect(),
            item => vec![item],
        }
        .into_iter()
    })
}
/// Parses a type alias into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
fn parse_type_alias(t: &ItemType, source: Source) -> Result<ParsedTypeAlias, ParseError> {
    let typeshare_attr = TypeShareAttrs::from_attrs(&t.attrs)?;

    let ty = if let Some(ty) = typeshare_attr.serialized_as {
        ty
    } else {
        ParseType::try_from(&*t.ty)?.into()
    };

    let generic_types = ParseGenerics::from_syn_generics(&t.generics).into();

    Ok(ParsedTypeAlias {
        source,
        id: get_ident(Some(&t.ident), None, None),
        value_type: ty,
        comments: Comment::Multiline {
            comment: parse_comment_attrs(&t.attrs),
            location: CommentLocation::Type,
        },
        generic_types,
    })
}

// Helpers

/// Parses any comment out of the given slice of attributes
pub fn parse_comment_attrs(attrs: &[Attribute]) -> Vec<String> {
    const DOC_ATTR: &str = "doc";
    attrs
        .iter()
        .map(|attr| &attr.meta)
        .filter_map(|attr| match attr {
            Meta::NameValue(name_value) => {
                if name_value.path.is_ident(DOC_ATTR) {
                    match &name_value.value {
                        Expr::Lit(lit) => literal_as_string(lit.lit.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .map(|string| string.trim().into())
        .collect()
}

/// Checks the given attrs for `#[typeshare]`
fn has_typeshare_annotation(attrs: &[Attribute]) -> bool {
    let typeshare_ident = Ident::new("typeshare", Span::call_site());
    for a in attrs {
        if let Some(segment) = a.path().segments.iter().next() {
            if segment.ident == typeshare_ident {
                return true;
            }
        }
    }

    false
}

fn get_ident(ident: Option<&Ident>, rename: Option<&String>, rename_all: Option<&RenameAll>) -> Id {
    let original = ident.map_or("???".to_string(), |id| id.to_string().replace("r#", ""));

    let mut renamed = rename_all_to_case(original.clone(), rename_all);

    if let Some(s) = rename {
        renamed = s.clone();
    }

    Id {
        original,
        renamed,
        rename_all: rename_all.cloned(),
    }
}

fn rename_all_to_case(original: String, case: Option<&RenameAll>) -> String {
    original.to_case_option(case.copied())
}

fn literal_as_string(lit: syn::Lit) -> Option<String> {
    match lit {
        syn::Lit::Str(str) => Some(str.value()),
        _ => None,
    }
}
