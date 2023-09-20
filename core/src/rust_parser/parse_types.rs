use crate::parsed_types::{Generics, Number, SpecialType, Type, TypeError};
use crate::rename::RenameAll;
use quote::ToTokens;
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprLit, GenericParam, Lit, LitStr, TypeArray, TypeSlice};

impl Parse for RenameAll {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<LitStr>()?;

        Self::from_str(&ident.value())
            .map_err(|_| syn::Error::new(ident.span(), "invalid rename_all value"))
    }
}
// TODO: parsing is very opinionated and makes some decisions that should be
// getting made at code generation time. Fix this.
impl Parse for Type {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            return Ok(Type::Simple {
                id: input.parse::<LitStr>()?.value(),
            });
        }
        return Err(syn::Error::new(input.span(), "expected a string literal"));
    }
}
impl Generics {
    pub fn from_syn_generics(generics: &syn::Generics) -> Self {
        let value = generics
            .params
            .iter()
            .filter_map(|param| match param {
                GenericParam::Type(type_param) => Some(type_param.ident.to_string()),
                _ => None,
            })
            .collect();

        Self { generics: value }
    }
}

impl FromStr for Type {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let syn_type = syn::parse_str(s).map_err(|_| TypeError::UnsupportedType(vec![]))?;
        Self::try_from(&syn_type)
    }
}

impl TryFrom<&syn::Type> for Type {
    type Error = TypeError;

    fn try_from(ty: &syn::Type) -> Result<Self, Self::Error> {
        Ok(match ty {
            syn::Type::Tuple(tuple) if tuple.elems.iter().count() == 0 => {
                Self::Special(SpecialType::Unit)
            }
            syn::Type::Tuple(_) => return Err(TypeError::UnexpectedParameterizedTuple),
            syn::Type::Reference(reference) => Self::try_from(reference.elem.as_ref())?,
            syn::Type::Path(path) => {
                let segment = path.path.segments.iter().last().unwrap();
                let id = segment.ident.to_string();
                let parameters: Vec<Self> = match &segment.arguments {
                    syn::PathArguments::AngleBracketed(angle_bracketed_arguments) => {
                        let parameters: Result<Vec<Self>, Self::Error> = angle_bracketed_arguments
                            .args
                            .iter()
                            .filter_map(|arg| match arg {
                                syn::GenericArgument::Type(r#type) => Some(Self::try_from(r#type)),
                                _ => None,
                            })
                            .collect();
                        parameters?
                    }
                    _ => Vec::default(),
                };
                match id.as_str() {
                    "Vec" => Self::Special(SpecialType::Vec(
                        parameters.into_iter().next().unwrap().into(),
                    )),
                    "Option" => Self::Special(SpecialType::Option(
                        parameters.into_iter().next().unwrap().into(),
                    )),
                    "HashMap" => {
                        let mut params = parameters.into_iter();
                        Self::Special(SpecialType::Map(
                            params.next().unwrap().into(),
                            params.next().unwrap().into(),
                        ))
                    }
                    "str" | "String" => Self::Special(SpecialType::String),
                    // Since we do not need to box types in other languages, we treat this type
                    // as its inner type.
                    "Box" => parameters.into_iter().next().unwrap(),
                    "bool" => Self::Special(SpecialType::Bool),
                    "char" => Self::Special(SpecialType::Char),
                    "u8" => Self::Special(Number::U8.into()),
                    "u16" => Self::Special(Number::U16.into()),
                    "u32" => Self::Special(Number::U32.into()),
                    "U53" => Self::Special(Number::U53.into()),
                    "u64" => Self::Special(Number::U64.into()),
                    "usize" => Self::Special(Number::USize.into()),
                    "i8" => Self::Special(Number::I8.into()),
                    "i16" => Self::Special(Number::I16.into()),
                    "i32" => Self::Special(Number::I32.into()),
                    "I54" => Self::Special(Number::I54.into()),
                    "i64" => Self::Special(Number::I64.into()),
                    "isize" => Self::Special(Number::ISize.into()),
                    "f32" => Self::Special(Number::F32.into()),
                    "f64" => Self::Special(Number::F64.into()),
                    _ => {
                        if parameters.is_empty() {
                            Self::Simple { id }
                        } else {
                            Self::Generic { id, parameters }
                        }
                    }
                }
            }
            syn::Type::Array(TypeArray {
                elem,
                len:
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(count),
                        ..
                    }),
                ..
            }) => Self::Special(SpecialType::Array(
                Self::try_from(elem.as_ref())?.into(),
                count.base10_parse().map_err(|v| {
                    TypeError::UnexpectedToken(format!("{} is not a valid array length", v))
                })?,
            )),
            syn::Type::Slice(TypeSlice {
                bracket_token: _,
                elem,
            }) => Self::Special(SpecialType::Slice(Self::try_from(elem.as_ref())?.into())),
            _ => return Err(TypeError::UnexpectedToken(ty.to_token_stream().to_string())),
        })
    }
}
