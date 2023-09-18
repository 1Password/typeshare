use crate::rename::RenameAll;
use itertools::Itertools;
use std::ops::Add;
use syn::parse::{Parse, ParseStream};
use syn::Attribute;

const SERDE: &str = "serde";
pub struct SerdeEnumAttrs {
    pub tag: String,
    pub content: String,
}
impl SerdeContainerAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        attrs
            .iter()
            .filter(|a| a.path().is_ident(SERDE))
            .map(|a| a.parse_args::<SerdeContainerAttrs>())
            .flatten_ok()
            .sum()
    }
}

pub struct SerdeContainerAttrs {
    pub rename_all: Option<RenameAll>,
    pub enum_attrs: Option<SerdeEnumAttrs>,

}
impl Add for SerdeContainerAttrs {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            rename_all: self.rename_all.or(rhs.rename_all),
            enum_attrs: self.enum_attrs.or(rhs.enum_attrs),
        }
    }
}
impl Parse for SerdeContainerAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {}
}

pub struct SerdeFieldAttrs {
    pub rename: Option<String>,
    pub flatten: bool,
    pub skip: bool,
    pub default: Option<String>,
}

impl Add for SerdeFieldAttrs {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            rename: self.rename.or(rhs.rename),
            flatten: self.flatten || rhs.flatten,
            skip: self.skip || rhs.skip,
        }
    }
}
impl SerdeFieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        attrs
            .iter()
            .filter(|a| a.path().is_ident(SERDE))
            .map(|a| a.parse_args::<SerdeFieldAttrs>())
            .flatten_ok()
            .sum()
    }
}
impl Parse for SerdeFieldAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {}
}
