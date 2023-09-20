mod decorator;
mod enum_parse;
/// Implements Parsing for some types found inside types module
mod parse_types;
pub(crate) mod pub_utils;
mod serde_parse;
mod struct_parse;
mod typeshare_attrs;

use crate::rust_parser::enum_parse::parse_enum;

use crate::parsed_types::{
    Comment, CommentLocation, Generics, ParsedData, ParsedTypeAlias, Source, TypeError,
};
use crate::rename::RenameAll;
use crate::rust_parser::struct_parse::parse_struct;
use crate::rust_parser::typeshare_attrs::TypeShareAttrs;
use crate::{
    parsed_types::{Id, Type},
    rename::RenameExt,
};
use proc_macro2::{Ident, Span};
use std::convert::TryFrom;

use syn::Meta;
use syn::{Attribute, Expr, ItemType};
use thiserror::Error;

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

/// Parse the given Rust source string into `ParsedData`.
pub fn parse(input: &str, source: Source) -> Result<ParsedData, ParseError> {
    let mut parsed_data = ParsedData::default();
    parse_into(input, &mut parsed_data, source)?;
    Ok(parsed_data)
}
pub fn parse_into(input: &str, target: &mut ParsedData, source: Source) -> Result<(), ParseError> {
    // We will only produce output for files that contain the `#[typeshare]`
    // attribute, so this is a quick and easy performance win
    if !input.contains("typeshare") {
        return Ok(());
    }

    // Parse and process the input, ensuring we parse only items marked with
    // `#[typeshare]
    let syn_file = syn::parse_file(input)?;

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
        Type::try_from(&*t.ty)?
    };

    let generic_types = Generics::from_syn_generics(&t.generics);

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
fn has_typeshare_annotation(attrs: &[syn::Attribute]) -> bool {
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

fn get_ident(
    ident: Option<&proc_macro2::Ident>,
    rename: Option<&String>,
    rename_all: Option<&RenameAll>,
) -> Id {
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
    match case {
        None => original,
        Some(value) => match value {
            RenameAll::CamelCase => original.to_camel_case(),
            RenameAll::PascalCase => original.to_pascal_case(),
            RenameAll::SnakeCase => original.to_snake_case(),
            RenameAll::ScreamingSnakeCase => original.to_screaming_snake_case(),
            RenameAll::KebabCase => original.to_kebab_case(),
            RenameAll::ScreamingKebabCase => original.to_screaming_kebab_case(),
        },
    }
}

fn literal_as_string(lit: syn::Lit) -> Option<String> {
    match lit {
        syn::Lit::Str(str) => Some(str.value()),
        _ => None,
    }
}
