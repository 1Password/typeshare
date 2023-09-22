use syn::{Fields, ItemStruct};

use crate::{
    parsed_types::{
        Comment, CommentLocation, Field, Generics, Item, ParsedStruct, ParsedTypeAlias, Source,
        StructShared, Type,
    },
    rust_parser::{
        decorator::get_lang_decorators,
        get_ident, parse_comment_attrs,
        serde_parse::{SerdeContainerAttrs, SerdeFieldAttrs},
        typeshare_attrs::{TypeShareAttrs, TypeShareFieldAttrs},
        ParseError,
    },
};

/// Parses a struct into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than a struct, which is a
/// hack.

pub fn parse_struct(s: &ItemStruct, source: Source) -> Result<Item, ParseError> {
    let typeshare_attr = TypeShareAttrs::from_attrs(&s.attrs)?;
    let serde_attrs = SerdeContainerAttrs::from_attrs(&s.attrs)?;
    let generic_types = Generics::from_syn_generics(&s.generics);
    // Check if this struct should be parsed as a type alias.
    // TODO: we shouldn't lie and return a type alias when parsing a struct. this
    // is a temporary hack
    if let Some(ty) = typeshare_attr.serialized_as {
        return Ok(Item::Alias(ParsedTypeAlias {
            source,
            id: get_ident(Some(&s.ident), None, None),
            value_type: ty,
            comments: Comment::Multiline {
                comment: parse_comment_attrs(&s.attrs),
                location: CommentLocation::Type,
            },
            generic_types,
        }));
    }

    Ok(match &s.fields {
        // Structs
        Fields::Named(f) => {
            let mut fields = Vec::new();

            for field in f.named.iter() {
                let typeshare_field_attr = TypeShareFieldAttrs::from_attrs(&field.attrs)?;
                let serde_field_attrs = SerdeFieldAttrs::from_attrs(&field.attrs)?;
                let ty = if let Some(ty) = typeshare_field_attr.serialized_as {
                    ty
                } else {
                    Type::try_from(&field.ty)?
                };
                let has_default = serde_field_attrs.default.is_some();
                let lang_decorators = get_lang_decorators(&field.attrs)?;
                let field = Field {
                    id: get_ident(
                        field.ident.as_ref(),
                        serde_field_attrs.rename.as_ref(),
                        serde_attrs.rename_all.as_ref(),
                    ),
                    ty,
                    comments: Comment::Multiline {
                        comment: parse_comment_attrs(&field.attrs),
                        location: CommentLocation::Field,
                    },
                    has_default,
                    lang_decorators,
                };
                fields.push(field);
            }

            Item::Struct(ParsedStruct::TraditionalStruct {
                fields,
                shared: StructShared {
                    source,
                    id: get_ident(Some(&s.ident), None, None),
                    generic_types,
                    comments: Comment::Multiline {
                        comment: parse_comment_attrs(&s.attrs),
                        location: CommentLocation::Type,
                    },
                    decorators: get_lang_decorators(&s.attrs)?,
                },
            })
        }
        // Tuple structs
        Fields::Unnamed(f) => {
            if f.unnamed.len() > 1 {
                return Err(ParseError::ComplexTupleStruct);
            }
            let f = f.unnamed.first().unwrap();
            let typeshare_field_attr = TypeShareFieldAttrs::from_attrs(&f.attrs)?;
            let ty = if let Some(ty) = typeshare_field_attr.serialized_as {
                ty
            } else {
                Type::try_from(&f.ty)?
            };

            Item::Alias(ParsedTypeAlias {
                source,
                id: get_ident(Some(&s.ident), None, None),
                value_type: ty,
                comments: Comment::Multiline {
                    comment: parse_comment_attrs(&s.attrs),
                    location: CommentLocation::Field,
                },
                generic_types,
            })
        }
        // Unit structs or `None`
        Fields::Unit => Item::Struct(ParsedStruct::TraditionalStruct {
            fields: vec![],
            shared: StructShared {
                source,
                id: get_ident(Some(&s.ident), None, None),
                generic_types,
                comments: Comment::Multiline {
                    comment: parse_comment_attrs(&s.attrs),
                    location: CommentLocation::Type,
                },
                decorators: get_lang_decorators(&s.attrs)?,
            },
        }),
    })
}
