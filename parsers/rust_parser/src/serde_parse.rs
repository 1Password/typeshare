use std::ops::Add;

use syn::{
    parse::{Parse, ParseStream},
    Attribute, Token,
};

use crate::parse_types::ParseRenameAll;
use typeshare_core::rename::RenameAll;

const SERDE: &str = "serde";
#[derive(Debug, Clone, PartialEq, Eq, Default)]

pub struct SerdeEnumAttrs {
    pub tag: Option<String>,
    pub content: Option<String>,
}
impl Add for SerdeEnumAttrs {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            tag: self.tag.or(rhs.tag),
            content: self.content.or(rhs.content),
        }
    }
}
impl SerdeContainerAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident(SERDE) {
                let parse = attr.parse_args::<SerdeContainerAttrs>()?;
                result = result + parse;
            }
        }
        Ok(result)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]

pub struct SerdeContainerAttrs {
    pub rename_all: Option<RenameAll>,
    pub enum_attrs: SerdeEnumAttrs,
}
impl Add for SerdeContainerAttrs {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            rename_all: self.rename_all.or(rhs.rename_all),
            enum_attrs: self.enum_attrs + rhs.enum_attrs,
        }
    }
}
impl Parse for SerdeContainerAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut rename_all = None;
        let mut enum_attrs = SerdeEnumAttrs {
            tag: None,
            content: None,
        };
        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "rename_all" => {
                    input.parse::<Token![=]>()?;
                    let rename_all_str = input.parse::<syn::LitStr>()?;
                    rename_all = Some(
                        rename_all_str
                            .value()
                            .parse::<RenameAll>()
                            .map_err(|e| syn::Error::new_spanned(rename_all_str, e))?,
                    );
                }
                "tag" => {
                    input.parse::<Token![=]>()?;
                    let tag_str = input.parse::<syn::LitStr>()?;
                    enum_attrs.tag = Some(tag_str.value());
                }
                "content" => {
                    input.parse::<Token![=]>()?;
                    let content_str = input.parse::<syn::LitStr>()?;
                    enum_attrs.content = Some(content_str.value());
                }
                _ => {}
            }
            if input.is_empty() {
                break;
            }
            let _ = input.parse::<Token![,]>();
        }
        Ok(Self {
            rename_all,
            enum_attrs,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]

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
            default: self.default.or(rhs.default),
        }
    }
}
impl SerdeFieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident(SERDE) {
                let parse = attr.parse_args::<SerdeFieldAttrs>()?;
                result = result + parse;
            }
        }
        Ok(result)
    }
}
impl Parse for SerdeFieldAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut rename = None;
        let mut flatten = false;
        let mut skip = false;
        let mut default = None;
        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "rename" => {
                    input.parse::<Token![=]>()?;
                    let rename_str = input.parse::<syn::LitStr>()?;
                    rename = Some(rename_str.value());
                }
                "flatten" => {
                    flatten = true;
                }
                "skip" => {
                    skip = true;
                }
                "default" => {
                    input.parse::<Token![=]>()?;
                    let default_str = input.parse::<syn::LitStr>()?;
                    default = Some(default_str.value());
                }
                _ => {
                    if input.peek(Token![=]) {
                        let _ = input.parse::<Token![=]>()?;
                        let _ = input.parse::<syn::LitStr>()?;
                    }
                }
            }
            if input.is_empty() {
                break;
            }
            let _ = input.parse::<Token![,]>();
        }
        Ok(Self {
            rename,
            flatten,
            skip,
            default,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]

pub struct SerdeVariantAttr {
    pub rename_all: Option<RenameAll>,
    pub rename: Option<String>,
}

impl Parse for SerdeVariantAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut rename_all = None;
        let mut rename = None;
        while !input.is_empty() {
            let ident = input.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "rename_all" => {
                    input.parse::<Token![=]>()?;
                    rename_all = Some(input.parse::<ParseRenameAll>()?.0)
                }
                "rename" => {
                    input.parse::<Token![=]>()?;
                    let rename_str = input.parse::<syn::LitStr>()?;
                    rename = Some(rename_str.value());
                }
                _ => {}
            }
            if input.is_empty() {
                break;
            }
            let _ = input.parse::<Token![,]>();
        }
        Ok(Self { rename_all, rename })
    }
}

impl Add for SerdeVariantAttr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            rename_all: self.rename_all.or(rhs.rename_all),
            rename: self.rename.or(rhs.rename),
        }
    }
}
impl SerdeVariantAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        let mut result = Self::default();

        for attr in attrs {
            if attr.path().is_ident(SERDE) {
                let parse = attr.parse_args::<SerdeVariantAttr>()?;
                result = result + parse;
            }
        }
        Ok(result)
    }
}
