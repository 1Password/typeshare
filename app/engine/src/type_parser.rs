use itertools::Itertools;
use quote::ToTokens;
use syn::Ident;
use typeshare_model::prelude::{RustType, SpecialRustType, TypeName};

use crate::{ParseError, ParseErrorKind, RustTypeParseError};

pub fn type_name(ident: &Ident) -> TypeName {
    TypeName::new_string(ident.to_string())
}

pub fn parse_rust_type(tokens: &syn::Type) -> Result<RustType, ParseError> {
    Ok(match tokens {
        syn::Type::Tuple(tuple) if tuple.elems.iter().count() == 0 => {
            RustType::Special(SpecialRustType::Unit)
        }
        syn::Type::Tuple(_) => {
            return Err(ParseError::new(
                tokens,
                ParseErrorKind::RustTypeParseError(
                    RustTypeParseError::UnexpectedParameterizedTuple,
                ),
            ))
        }
        syn::Type::Reference(reference) => parse_rust_type(&reference.elem)?,
        syn::Type::Path(path) => {
            let segment = path.path.segments.iter().last().unwrap();
            let id = type_name(&segment.ident);
            let parameters: Vec<RustType> = match &segment.arguments {
                syn::PathArguments::AngleBracketed(angle_bracketed_arguments) => {
                    let parameters: Result<Vec<RustType>, ParseError> = angle_bracketed_arguments
                        .args
                        .iter()
                        .filter_map(|arg| match arg {
                            syn::GenericArgument::Type(ty) => Some(parse_rust_type(ty)),
                            _ => None,
                        })
                        .collect();
                    parameters?
                }
                _ => Vec::default(),
            };
            match id.as_str() {
                "Vec" => RustType::Special(SpecialRustType::Vec(Box::new(
                    parameters
                        .into_iter()
                        .exactly_one()
                        .expect("vec with wrong number of types"),
                ))),
                "Option" => RustType::Special(SpecialRustType::Option(Box::new(
                    parameters
                        .into_iter()
                        .exactly_one()
                        .expect("option with wrong number of types"),
                ))),
                "HashMap" => {
                    let mut params = parameters.into_iter();
                    RustType::Special(SpecialRustType::HashMap(
                        params.next().unwrap().into(),
                        params.exactly_one().unwrap().into(),
                    ))
                }
                "str" | "String" => RustType::Special(SpecialRustType::String),
                // These smart pointers can be treated as their inner type since serde can handle it
                // See impls of serde::Deserialize
                "Box" | "Weak" | "Arc" | "Rc" | "Cow" | "ArcWeak" | "RcWeak" | "Cell" | "Mutex"
                | "RefCell" | "RwLock" => parameters.into_iter().next().unwrap(),
                "bool" => RustType::Special(SpecialRustType::Bool),
                "char" => RustType::Special(SpecialRustType::Char),
                "u8" => RustType::Special(SpecialRustType::U8),
                "u16" => RustType::Special(SpecialRustType::U16),
                "u32" => RustType::Special(SpecialRustType::U32),
                "U53" => RustType::Special(SpecialRustType::U53),
                id @ ("u64" | "i64" | "usize" | "isize") => {
                    return Err(ParseError::new(
                        &segment.ident,
                        ParseErrorKind::RustTypeParseError(RustTypeParseError::UnsupportedType(
                            vec![id.to_owned()],
                        )),
                    ))
                }
                "i8" => RustType::Special(SpecialRustType::I8),
                "i16" => RustType::Special(SpecialRustType::I16),
                "i32" => RustType::Special(SpecialRustType::I32),
                "I54" => RustType::Special(SpecialRustType::I54),
                "f32" => RustType::Special(SpecialRustType::F32),
                "f64" => RustType::Special(SpecialRustType::F64),
                _ => {
                    if parameters.is_empty() {
                        RustType::Simple { id }
                    } else {
                        RustType::Generic { id, parameters }
                    }
                }
            }
        }
        syn::Type::Array(syn::TypeArray {
            elem,
            len:
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(count),
                    ..
                }),
            ..
        }) => RustType::Special(SpecialRustType::Array(
            Box::new(parse_rust_type(elem)?),
            count.base10_parse().map_err(|err| {
                ParseError::new(
                    count,
                    ParseErrorKind::RustTypeParseError(RustTypeParseError::NumericLiteral(err)),
                )
            })?,
        )),
        syn::Type::Slice(syn::TypeSlice {
            bracket_token: _,
            elem,
        }) => RustType::Special(SpecialRustType::Slice(Box::new(parse_rust_type(elem)?))),
        _ => {
            return Err(ParseError::new(
                &tokens,
                ParseErrorKind::RustTypeParseError(RustTypeParseError::UnexpectedToken(
                    tokens.to_token_stream().to_string(),
                )),
            ))
        }
    })
}

// NOTE: try to avoid using this, if you can, in favor of `parse_rust_type`
pub fn parse_rust_type_from_string(input: &str) -> Result<RustType, ParseError> {
    parse_rust_type(
        &syn::parse_str(input)
            .map_err(|err| ParseError::new(&err.span(), ParseErrorKind::SynError(err)))?,
    )
}
