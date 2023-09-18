use crate::parser::typeshare_attrs::{TypeShareAttrs, TypeShareFieldAttrs};
use crate::parser::{get_ident, parse_comment_attrs, ParseError};
use crate::rust_types::{RustField, RustItem, RustStruct, RustType, RustTypeAlias};
use syn::{Fields, GenericParam, ItemStruct};
use crate::parser::decorator::get_lang_decorators;
use crate::parser::serde_parse::{SerdeContainerAttrs, SerdeFieldAttrs};

/// Parses a struct into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than a struct, which is a
/// hack.
///

pub fn parse_struct(s: &ItemStruct) -> Result<RustItem, ParseError> {
    let typeshare_attr = TypeShareAttrs::from_attrs(&s.attrs)?;
    let serde_attrs = SerdeContainerAttrs::from_attrs(&s.attrs)?;

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
    if let Some(ty) = typeshare_attr.serialized_as {
        return Ok(RustItem::Alias(RustTypeAlias {
            id: get_ident(Some(&s.ident), None, &None),
            r#type: ty,
            comments: parse_comment_attrs(&s.attrs),
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
                    RustType::try_from(&field.ty)?
                };
                let has_default = serde_field_attrs.default.is_some();
                let lang_decorators = get_lang_decorators(&field.attrs);
                let field = RustField {
                    id: get_ident(field.ident.as_ref(), Some(&serde_field_attrs), &serde_attrs.rename_all),
                    ty,
                    comments: parse_comment_attrs(&field.attrs),
                    has_default,
                    lang_decorators,
                };
                fields.push(field);
            }


            RustItem::Struct(RustStruct {
                id: get_ident(Some(&s.ident), None, &None),
                generic_types,
                fields,
                comments: parse_comment_attrs(&s.attrs),
                decorators: get_lang_decorators(&s.attrs),
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
                RustType::try_from(&f.ty)?
            };

            RustItem::Alias(RustTypeAlias {
                id: get_ident(Some(&s.ident), None, &None),
                r#type: ty,
                comments: parse_comment_attrs(&s.attrs),
                generic_types,
            })
        }
        // Unit structs or `None`
        Fields::Unit => RustItem::Struct(RustStruct {
            id: get_ident(Some(&s.ident), None, &None),
            generic_types,
            fields: vec![],
            comments: parse_comment_attrs(&s.attrs),
            decorators: get_lang_decorators(&s.attrs),
        }),
    })
}
