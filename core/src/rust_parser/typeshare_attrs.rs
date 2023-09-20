use crate::rust_parser::TYPESHARE_ATTR;

use crate::parsed_types::Type;

use proc_macro2::TokenStream;
use std::ops::Add;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Meta, Token};
mod keywords {
    syn::custom_keyword!(rename);
    syn::custom_keyword!(serialized_as);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(lang);
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeShareAttrs {
    pub serialized_as: Option<Type>,
}
impl TypeShareAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident(TYPESHARE_ATTR) {
                if let Meta::List(_) = &attr.meta {
                    let attr_result = attr.parse_args()?;
                    result = result + attr_result;
                }
            }
        }
        Ok(result)
    }
}
impl Add for TypeShareAttrs {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            serialized_as: self.serialized_as.or(rhs.serialized_as),
        }
    }
}
impl Parse for TypeShareAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut serialized_as = None;

        while input.peek(syn::Ident) {
            let ident = input.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "serialized_as" => {
                    input.parse::<Token![=]>()?;
                    let ty = input.parse::<Type>()?;
                    serialized_as = Some(ty);
                }
                "lang" => {
                    return Ok(Self::default());
                }
                _ => {}
            }
            if input.is_empty() {
                break;
            }
            let _ = input.parse::<Token![,]>();
        }
        Ok(Self { serialized_as })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]

pub struct TypeShareFieldAttrs {
    pub rename: Option<String>,
    pub serialized_as: Option<Type>,
    pub skip: bool,
}
impl TypeShareFieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident(TYPESHARE_ATTR) {
                let parse = attr.parse_args::<TypeShareFieldAttrs>()?;
                result = result + parse;
            }
        }
        Ok(result)
    }
}
impl Add for TypeShareFieldAttrs {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            rename: self.rename.or(rhs.rename),
            serialized_as: self.serialized_as.or(rhs.serialized_as),
            skip: self.skip || rhs.skip,
        }
    }
}
impl Parse for TypeShareFieldAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut rename = None;
        let mut serialized_as = None;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(keywords::rename) {
                input.parse::<keywords::rename>()?;
                input.parse::<Token![=]>()?;
                let rename_str = input.parse::<syn::LitStr>()?;
                rename = Some(rename_str.value());
            } else if lookahead.peek(keywords::serialized_as) {
                input.parse::<keywords::serialized_as>()?;
                input.parse::<Token![=]>()?;
                let ty = input.parse::<Type>()?;
                serialized_as = Some(ty);
            } else if lookahead.peek(keywords::skip) {
                input.parse::<keywords::skip>()?;
                return Ok(Self {
                    rename: None,
                    serialized_as: None,
                    skip: true,
                });
            } else if lookahead.peek(keywords::lang) {
                let _ = input.parse::<TokenStream>();
                return Ok(Self::default());
            } else {
                return Err(lookahead.error());
            }
            let _ = input.parse::<Token![,]>();
        }
        Ok(Self {
            rename,
            serialized_as,
            skip: false,
        })
    }
}
