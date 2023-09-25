use derive_more::{Deref, From, Into};
use std::str::FromStr;

use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Expr, ExprLit, GenericParam, Lit, LitStr, TypeArray, TypeSlice,
};

use typeshare_core::{
    parsed_types::{Generics, Number, SpecialType, Type, TypeError},
    rename::RenameAll,
};

#[derive(Debug, Clone, From, Into, Deref)]
pub struct ParseRenameAll(pub RenameAll);
#[derive(Debug, Clone, From, Into, Deref)]
pub struct ParseGenerics(pub Generics);
#[derive(Debug, Clone, From, Into, Deref)]
pub struct ParseType(pub Type);

impl Parse for ParseRenameAll {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<LitStr>()?;

        RenameAll::from_str(&ident.value())
            .map(Self)
            .map_err(|_| syn::Error::new(ident.span(), "invalid rename_all value"))
    }
}
// TODO: parsing is very opinionated and makes some decisions that should be
// getting made at code generation time. Fix this.
impl Parse for ParseType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            return Ok(ParseType(Type::Simple {
                id: input.parse::<LitStr>()?.value(),
            }));
        }
        Err(syn::Error::new(input.span(), "expected a string literal"))
    }
}
impl ParseGenerics {
    pub fn from_syn_generics(generics: &syn::Generics) -> Self {
        let value = generics
            .params
            .iter()
            .filter_map(|param| match param {
                GenericParam::Type(type_param) => Some(type_param.ident.to_string()),
                _ => None,
            })
            .collect();

        Self(Generics { generics: value })
    }
}

impl FromStr for ParseType {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let syn_type = syn::parse_str(s).map_err(|_| TypeError::UnsupportedType(vec![]))?;
        Self::try_from(&syn_type)
    }
}

impl TryFrom<&syn::Type> for ParseType {
    type Error = TypeError;

    fn try_from(ty: &syn::Type) -> Result<Self, Self::Error> {
        Ok(match ty {
            syn::Type::Tuple(tuple) if tuple.elems.iter().count() == 0 => {
                Self(Type::Special(SpecialType::Unit))
            }
            syn::Type::Tuple(_) => return Err(TypeError::UnexpectedParameterizedTuple),
            syn::Type::Reference(reference) => Self::try_from(reference.elem.as_ref())?,
            syn::Type::Path(path) => {
                let segment = path.path.segments.iter().last().unwrap();
                let id = segment.ident.to_string();
                let parameters: Vec<Type> = match &segment.arguments {
                    syn::PathArguments::AngleBracketed(angle_bracketed_arguments) => {
                        let parameters: Result<Vec<Self>, Self::Error> = angle_bracketed_arguments
                            .args
                            .iter()
                            .filter_map(|arg| match arg {
                                syn::GenericArgument::Type(r#type) => Some(Self::try_from(r#type)),
                                _ => None,
                            })
                            .collect();
                        parameters?.into_iter().map(|v| v.0).collect()
                    }
                    _ => Vec::default(),
                };
                match id.as_str() {
                    "Vec" => Type::Special(SpecialType::Vec(
                        parameters.into_iter().next().unwrap().into(),
                    )),
                    "Option" => Type::Special(SpecialType::Option(
                        parameters.into_iter().next().unwrap().into(),
                    )),
                    "HashMap" => {
                        let mut params = parameters.into_iter();
                        Type::Special(SpecialType::Map(
                            params.next().unwrap().into(),
                            params.next().unwrap().into(),
                        ))
                    }
                    "str" | "String" => Type::Special(SpecialType::String),
                    // Since we do not need to box types in other languages, we treat this type
                    // as its inner type.
                    "Box" => parameters.into_iter().next().unwrap(),
                    "bool" => Type::Special(SpecialType::Bool),
                    "char" => Type::Special(SpecialType::Char),
                    "u8" => Type::Special(Number::U8.into()),
                    "u16" => Type::Special(Number::U16.into()),
                    "u32" => Type::Special(Number::U32.into()),
                    "U53" => Type::Special(Number::U53.into()),
                    "u64" => Type::Special(Number::U64.into()),
                    "usize" => Type::Special(Number::USize.into()),
                    "i8" => Type::Special(Number::I8.into()),
                    "i16" => Type::Special(Number::I16.into()),
                    "i32" => Type::Special(Number::I32.into()),
                    "I54" => Type::Special(Number::I54.into()),
                    "i64" => Type::Special(Number::I64.into()),
                    "isize" => Type::Special(Number::ISize.into()),
                    "f32" => Type::Special(Number::F32.into()),
                    "f64" => Type::Special(Number::F64.into()),
                    _ => {
                        if parameters.is_empty() {
                            Type::Simple { id }
                        } else {
                            Type::Generic { id, parameters }
                        }
                    }
                }
                .into()
            }
            syn::Type::Array(TypeArray {
                elem,
                len:
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(count),
                        ..
                    }),
                ..
            }) => Type::Special(SpecialType::Array(
                Box::new(Self::try_from(elem.as_ref())?.into()),
                count.base10_parse().map_err(|v| {
                    TypeError::UnexpectedToken(format!("{} is not a valid array length", v))
                })?,
            ))
            .into(),
            syn::Type::Slice(TypeSlice {
                bracket_token: _,
                elem,
            }) => Type::Special(SpecialType::Slice(Box::new(
                Self::try_from(elem.as_ref())?.into(),
            )))
            .into(),
            _ => return Err(TypeError::UnexpectedToken(ty.to_token_stream().to_string())),
        })
    }
}
