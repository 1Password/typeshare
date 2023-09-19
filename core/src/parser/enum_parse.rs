use crate::language::{Comment, CommentLocation};
use crate::parser::decorator::get_lang_decorators;
use crate::parser::serde_parse::{SerdeContainerAttrs, SerdeFieldAttrs, SerdeVariantAttr};
use crate::parser::typeshare_attrs::{TypeShareAttrs, TypeShareFieldAttrs};
use crate::parser::{get_ident, parse_comment_attrs, ParseError};
use crate::rename::RenameAll;
use crate::rust_types::{
    Generics, RustEnum, RustEnumShared, RustEnumVariant, RustEnumVariantShared, RustField,
    RustItem, RustType, RustTypeAlias, Source,
};
use syn::ItemEnum;

/// Parses an enum into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than an enum, which is a
/// hack.
pub fn parse_enum(e: &ItemEnum, source: Source) -> Result<RustItem, ParseError> {
    let generic_types = Generics::from_syn_generics(&e.generics);
    let typeshare_attr = TypeShareAttrs::from_attrs(&e.attrs)?;
    let serde_attrs = SerdeContainerAttrs::from_attrs(&e.attrs)?;

    // TODO: we shouldn't lie and return a type alias when parsing an enum. this
    // is a temporary hack
    if let Some(ty) = typeshare_attr.serialized_as {
        return Ok(RustItem::Alias(RustTypeAlias {
            source,
            id: get_ident(Some(&e.ident), None, None),
            r#type: ty,
            comments: Comment::MultilineOwned {
                comment: parse_comment_attrs(&e.attrs),
                location: CommentLocation::Type,
            },
            generic_types,
        }));
    }

    let original_enum_ident = e.ident.to_string();
    let mut variants = Vec::new();
    // Parse all of the enum's variants
    for variant in &e.variants {
        let variant = parse_enum_variant(variant, &serde_attrs.rename_all)?;
        variants.push(variant);
    }

    // Check if the enum references itself recursively in any of its variants
    let is_recursive = variants.iter().any(|v| match v {
        RustEnumVariant::Unit(_) => false,
        RustEnumVariant::Tuple { ty, .. } => ty.contains_type(&original_enum_ident),
        RustEnumVariant::AnonymousStruct { fields, .. } => fields
            .iter()
            .any(|f| f.ty.contains_type(&original_enum_ident)),
    });

    let shared = RustEnumShared {
        source,
        id: get_ident(Some(&e.ident), None, None),
        comments: Comment::MultilineOwned {
            comment: parse_comment_attrs(&e.attrs),
            location: CommentLocation::Type,
        },
        variants,
        lang_decorators: get_lang_decorators(&e.attrs)?,
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

        if serde_attrs.enum_attrs.tag.is_some() {
            return Err(ParseError::SerdeTagNotAllowed {
                enum_ident: original_enum_ident,
            });
        }
        if serde_attrs.enum_attrs.content.is_some() {
            return Err(ParseError::SerdeContentNotAllowed {
                enum_ident: original_enum_ident,
            });
        }

        Ok(RustItem::Enum(RustEnum::Unit(shared)))
    } else {
        // At least one enum variant is either a tuple or an anonymous struct

        let tag_key = serde_attrs
            .enum_attrs
            .tag
            .ok_or_else(|| ParseError::SerdeTagRequired {
                enum_ident: original_enum_ident.clone(),
            })?;
        let content_key =
            serde_attrs
                .enum_attrs
                .content
                .ok_or_else(|| ParseError::SerdeContentRequired {
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
    enum_serde_rename_all: &Option<RenameAll>,
) -> Result<RustEnumVariant, ParseError> {
    let typeshare_variant_attr = TypeShareFieldAttrs::from_attrs(&v.attrs)?;
    let serde_variant_attrs = SerdeVariantAttr::from_attrs(&v.attrs)?;
    let shared = RustEnumVariantShared {
        id: get_ident(
            Some(&v.ident),
            serde_variant_attrs.rename.as_ref(),
            enum_serde_rename_all.as_ref(),
        ),
        comments: Comment::MultilineOwned {
            comment: parse_comment_attrs(&v.attrs),
            location: CommentLocation::Field,
        },
    };

    match &v.fields {
        syn::Fields::Unit => Ok(RustEnumVariant::Unit(shared)),
        syn::Fields::Unnamed(associated_type) => {
            if associated_type.unnamed.len() > 1 {
                return Err(ParseError::MultipleUnnamedAssociatedTypes);
            }
            let first_field = associated_type.unnamed.first().unwrap();

            let ty = if let Some(ty) = typeshare_variant_attr.serialized_as {
                ty
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
                    let typeshare_field_attr = TypeShareFieldAttrs::from_attrs(&f.attrs)?;
                    let serde_field_attrs = SerdeFieldAttrs::from_attrs(&f.attrs)?;
                    let field_type = if let Some(ty) = typeshare_field_attr.serialized_as {
                        ty
                    } else {
                        RustType::try_from(&f.ty)?
                    };
                    let lang_decorators = get_lang_decorators(&f.attrs)?;
                    Ok(RustField {
                        id: get_ident(
                            f.ident.as_ref(),
                            serde_field_attrs.rename.as_ref(),
                            serde_variant_attrs.rename_all.as_ref(),
                        ),
                        ty: field_type,
                        comments: Comment::MultilineOwned {
                            comment: parse_comment_attrs(&f.attrs),
                            location: CommentLocation::Field,
                        },
                        has_default: serde_field_attrs.default.is_some(),
                        lang_decorators,
                    })
                })
                .collect::<Result<Vec<_>, ParseError>>()?,
            shared,
        }),
    }
}
