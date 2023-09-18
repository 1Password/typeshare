use crate::parser::TYPESHARE_ATTR;
use crate::rename::RenameAll;
use crate::rust_types::RustType;
use itertools::Itertools;
use std::ops::Add;
use syn::parse::{Parse, ParseStream};
use syn::Attribute;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeShareAttrs {
    pub serialized_as: Option<RustType>,
}
impl TypeShareAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        attrs
            .iter()
            .filter(|a| a.path().is_ident(TYPESHARE_ATTR))
            .map(|a| a.parse_args::<TypeShareAttrs>())
            .flatten_ok()
            .sum()
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
    fn parse(input: ParseStream) -> syn::Result<Self> {}
}
#[derive(Debug, Clone, PartialEq, Eq)]

pub struct TypeShareFieldAttrs {
    pub rename: Option<String>,
    pub serialized_as: Option<RustType>,
    pub skip: bool,
}
impl TypeShareFieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        attrs
            .iter()
            .filter(|a| a.path().is_ident(TYPESHARE_ATTR))
            .map(|a| a.parse_args::<TypeShareFieldAttrs>())
            .flatten_ok()
            .sum()
    }
}
impl Add for TypeShareFieldAttrs {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            rename: self.rename.or(rhs.rename),
            skip: self.skip || rhs.skip,
        }
    }
}
impl Parse for TypeShareFieldAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {}
}
