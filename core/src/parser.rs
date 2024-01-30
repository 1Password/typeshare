use crate::helpers::{
    get_content_key, get_decorators, get_field_decorators, get_field_type_override, get_ident,
    get_serialized_as_type, get_tag_key, has_typeshare_annotation, is_skipped, parse_comment_attrs,
    serde_default, serde_flatten, serde_rename_all,
};

use crate::rust_types::{
    RustEnum, RustEnumShared, RustEnumVariant, RustEnumVariantShared, RustField, RustItem,
    RustStruct, RustType, RustTypeAlias, RustTypeParseError,
};

use std::convert::TryFrom;

use syn::GenericParam;
use syn::{Fields, Item, ItemEnum, ItemStruct, ItemType};
use thiserror::Error;

const TYPESHARE: &str = "typeshare";

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

/// The results of parsing Rust source input.
#[derive(Default, Debug)]
pub struct ParsedData {
    /// Structs defined in the source
    pub structs: Vec<RustStruct>,
    /// Enums defined in the source
    pub enums: Vec<RustEnum>,
    /// Type aliases defined in the source
    pub aliases: Vec<RustTypeAlias>,
}

impl ParsedData {
    /// Add the parsed data from `other` to `self`.
    pub fn add(&mut self, mut other: Self) {
        self.structs.append(&mut other.structs);
        self.enums.append(&mut other.enums);
        self.aliases.append(&mut other.aliases);
    }

    fn push(&mut self, rust_thing: RustItem) {
        match rust_thing {
            RustItem::Struct(s) => self.structs.push(s),
            RustItem::Enum(e) => self.enums.push(e),
            RustItem::Alias(a) => self.aliases.push(a),
        }
    }

    fn parse(&mut self, item: &Item) -> Result<(), ParseError> {
        match item {
            syn::Item::Struct(s) if has_typeshare_annotation(&s.attrs) => {
                self.push(parse_struct(s)?);
            }
            syn::Item::Enum(e) if has_typeshare_annotation(&e.attrs) => {
                self.push(parse_enum(e)?);
            }
            syn::Item::Type(t) if has_typeshare_annotation(&t.attrs) => {
                self.aliases.push(parse_type_alias(t)?);
            }
            _ => {}
        }

        Ok(())
    }
}

/// Parse the given Rust source string into `ParsedData`.
pub fn parse(input: &str) -> Result<ParsedData, ParseError> {
    let mut parsed_data = ParsedData::default();

    // We will only produce output for files that contain the `#[typeshare]`
    // attribute, so this is a quick and easy performance win
    if !input.contains(TYPESHARE) {
        return Ok(parsed_data);
    }

    // Parse and process the input, ensuring we parse only items marked with
    // `#[typeshare]`
    let source = syn::parse_file(input)?;

    for item in flatten_items(source.items.iter()) {
        parsed_data.parse(item)?;
    }

    Ok(parsed_data)
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

/// Parses a struct into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than a struct, which is a
/// hack.
fn parse_struct(s: &ItemStruct) -> Result<RustItem, ParseError> {
    let serde_rename_all = serde_rename_all(&s.attrs);

    let generic_types = s
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(type_param) => Some(type_param.ident.to_string()),
            _ => None,
        })
        .collect();

    // Check if this struct should be parsed as a type alias.
    // TODO: we shouldn't lie and return a type alias when parsing a struct. this
    // is a temporary hack
    if let Some(ty) = get_serialized_as_type(&s.attrs) {
        return Ok(RustItem::Alias(RustTypeAlias {
            id: get_ident(Some(&s.ident), &s.attrs, &None),
            r#type: ty.parse()?,
            comments: parse_comment_attrs(&s.attrs),
            generic_types,
        }));
    }

    Ok(match &s.fields {
        // Structs
        Fields::Named(f) => {
            let fields = f
                .named
                .iter()
                .filter(|field| !is_skipped(&field.attrs))
                .map(|f| {
                    let ty = if let Some(ty) = get_field_type_override(&f.attrs) {
                        ty.parse()?
                    } else {
                        RustType::try_from(&f.ty)?
                    };

                    if serde_flatten(&f.attrs) {
                        return Err(ParseError::SerdeFlattenNotAllowed);
                    }

                    let has_default = serde_default(&f.attrs);
                    let decorators = get_field_decorators(&f.attrs);

                    Ok(RustField {
                        id: get_ident(f.ident.as_ref(), &f.attrs, &serde_rename_all),
                        ty,
                        comments: parse_comment_attrs(&f.attrs),
                        has_default,
                        decorators,
                    })
                })
                .collect::<Result<_, ParseError>>()?;

            RustItem::Struct(RustStruct {
                id: get_ident(Some(&s.ident), &s.attrs, &None),
                generic_types,
                fields,
                comments: parse_comment_attrs(&s.attrs),
                decorators: get_decorators(&s.attrs),
            })
        }
        // Tuple structs
        Fields::Unnamed(f) => {
            if f.unnamed.len() > 1 {
                return Err(ParseError::ComplexTupleStruct);
            }
            let f = &f.unnamed[0];

            let ty = if let Some(ty) = get_field_type_override(&f.attrs) {
                ty.parse()?
            } else {
                RustType::try_from(&f.ty)?
            };

            RustItem::Alias(RustTypeAlias {
                id: get_ident(Some(&s.ident), &s.attrs, &None),
                r#type: ty,
                comments: parse_comment_attrs(&s.attrs),
                generic_types,
            })
        }
        // Unit structs or `None`
        Fields::Unit => RustItem::Struct(RustStruct {
            id: get_ident(Some(&s.ident), &s.attrs, &None),
            generic_types,
            fields: vec![],
            comments: parse_comment_attrs(&s.attrs),
            decorators: get_decorators(&s.attrs),
        }),
    })
}

/// Parses an enum into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than an enum, which is a
/// hack.
fn parse_enum(e: &ItemEnum) -> Result<RustItem, ParseError> {
    let generic_types = e
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(type_param) => Some(type_param.ident.to_string()),
            _ => None,
        })
        .collect();

    let serde_rename_all = serde_rename_all(&e.attrs);

    // TODO: we shouldn't lie and return a type alias when parsing an enum. this
    // is a temporary hack
    if let Some(ty) = get_serialized_as_type(&e.attrs) {
        return Ok(RustItem::Alias(RustTypeAlias {
            id: get_ident(Some(&e.ident), &e.attrs, &None),
            r#type: ty.parse()?,
            comments: parse_comment_attrs(&e.attrs),
            generic_types,
        }));
    }

    let original_enum_ident = e.ident.to_string();

    // Grab the `#[serde(tag = "...", content = "...")]` values if they exist
    let maybe_tag_key = get_tag_key(&e.attrs);
    let maybe_content_key = get_content_key(&e.attrs);

    // Parse all of the enum's variants
    let variants = e
        .variants
        .iter()
        // Filter out variants we've been told to skip
        .filter(|v| !is_skipped(&v.attrs))
        .map(|v| parse_enum_variant(v, &serde_rename_all))
        .collect::<Result<Vec<_>, _>>()?;

    // Check if the enum references itself recursively in any of its variants
    let is_recursive = variants.iter().any(|v| match v {
        RustEnumVariant::Unit(_) => false,
        RustEnumVariant::Tuple { ty, .. } => ty.contains_type(&original_enum_ident),
        RustEnumVariant::AnonymousStruct { fields, .. } => fields
            .iter()
            .any(|f| f.ty.contains_type(&original_enum_ident)),
    });

    let shared = RustEnumShared {
        id: get_ident(Some(&e.ident), &e.attrs, &None),
        comments: parse_comment_attrs(&e.attrs),
        variants,
        decorators: get_decorators(&e.attrs),
        generic_types,
        is_recursive,
    };

    // Figure out if we're dealing with a unit enum or an algebraic enum
    if shared
        .variants
        .iter()
        .all(|v| matches!(v, RustEnumVariant::Unit(_)))
    {
        // All enum variants are unit-type

        if maybe_tag_key.is_some() {
            return Err(ParseError::SerdeTagNotAllowed {
                enum_ident: original_enum_ident,
            });
        }
        if maybe_content_key.is_some() {
            return Err(ParseError::SerdeContentNotAllowed {
                enum_ident: original_enum_ident,
            });
        }

        Ok(RustItem::Enum(RustEnum::Unit(shared)))
    } else {
        // At least one enum variant is either a tuple or an anonymous struct

        let tag_key = maybe_tag_key.ok_or_else(|| ParseError::SerdeTagRequired {
            enum_ident: original_enum_ident.clone(),
        })?;
        let content_key = maybe_content_key.ok_or_else(|| ParseError::SerdeContentRequired {
            enum_ident: original_enum_ident.clone(),
        })?;

        Ok(RustItem::Enum(RustEnum::Algebraic {
            tag_key,
            content_key,
            shared,
        }))
    }
}

/// Parse an enum variant.
fn parse_enum_variant(
    v: &syn::Variant,
    enum_serde_rename_all: &Option<String>,
) -> Result<RustEnumVariant, ParseError> {
    let shared = RustEnumVariantShared {
        id: get_ident(Some(&v.ident), &v.attrs, enum_serde_rename_all),
        comments: parse_comment_attrs(&v.attrs),
    };

    // Get the value of `#[serde(rename_all)]` for this specific variant rather
    // than the overall enum
    //
    // The value of the attribute for the enum overall does not apply to enum
    // variant fields.
    let variant_serde_rename_all = serde_rename_all(&v.attrs);

    match &v.fields {
        syn::Fields::Unit => Ok(RustEnumVariant::Unit(shared)),
        syn::Fields::Unnamed(associated_type) => {
            if associated_type.unnamed.len() > 1 {
                return Err(ParseError::MultipleUnnamedAssociatedTypes);
            }

            let first_field = associated_type.unnamed.first().unwrap();

            let ty = if let Some(ty) = get_field_type_override(&first_field.attrs) {
                ty.parse()?
            } else {
                RustType::try_from(&first_field.ty)?
            };

            Ok(RustEnumVariant::Tuple { ty, shared })
        }
        syn::Fields::Named(fields_named) => Ok(RustEnumVariant::AnonymousStruct {
            fields: fields_named
                .named
                .iter()
                .map(|f| {
                    let field_type = if let Some(ty) = get_field_type_override(&f.attrs) {
                        ty.parse()?
                    } else {
                        RustType::try_from(&f.ty)?
                    };

                    let has_default = serde_default(&f.attrs);
                    let decorators = get_field_decorators(&f.attrs);

                    Ok(RustField {
                        id: get_ident(f.ident.as_ref(), &f.attrs, &variant_serde_rename_all),
                        ty: field_type,
                        comments: parse_comment_attrs(&f.attrs),
                        has_default,
                        decorators,
                    })
                })
                .collect::<Result<Vec<_>, ParseError>>()?,
            shared,
        }),
    }
}

/// Parses a type alias into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
fn parse_type_alias(t: &ItemType) -> Result<RustTypeAlias, ParseError> {
    let ty = if let Some(ty) = get_serialized_as_type(&t.attrs) {
        ty.parse()?
    } else {
        RustType::try_from(t.ty.as_ref())?
    };

    let generic_types = t
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(type_param) => Some(type_param.ident.to_string()),
            _ => None,
        })
        .collect();

    Ok(RustTypeAlias {
        id: get_ident(Some(&t.ident), &t.attrs, &None),
        r#type: ty,
        comments: parse_comment_attrs(&t.attrs),
        generic_types,
    })
}
