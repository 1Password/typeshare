use crate::parser::TYPESHARE_ATTR;

use crate::rust_types::RustType;

use proc_macro2::TokenStream;
use std::ops::Add;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Meta, Token};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeShareAttrs {
    pub serialized_as: Option<RustType>,
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
                    let ty = input.parse::<RustType>()?;
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
    pub serialized_as: Option<RustType>,
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
        let mut skip = false;

        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "rename" => {
                    input.parse::<Token![=]>()?;
                    let rename_str = input.parse::<syn::LitStr>()?;
                    rename = Some(rename_str.value());
                }
                "serialized_as" => {
                    input.parse::<Token![=]>()?;
                    let ty = input.parse::<RustType>()?;
                    serialized_as = Some(ty);
                }
                "skip" => {
                    skip = true;
                }
                "lang" => {
                    let _ = input.parse::<TokenStream>();
                    return Ok(Self::default());
                }
                _ => {}
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(Self {
            rename,
            serialized_as,
            skip,
        })
    }
}
