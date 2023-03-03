use crate::{
    rename::RenameExt,
    rust_types::{
        Id, RustEnum, RustEnumShared, RustEnumVariant, RustEnumVariantShared, RustField,
        RustStruct, RustType, RustTypeAlias, RustTypeParseError,
    },
};
use proc_macro2::{Ident, Span};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};
use syn::{Attribute, Fields, ItemEnum, ItemStruct, ItemType};
use syn::{GenericParam, Meta, NestedMeta};
use thiserror::Error;

// TODO: parsing is very opinionated and makes some decisions that should be
// getting made at code generation time. Fix this.

const SERDE: &str = "serde";
const TYPESHARE: &str = "typeshare";

/// The results of parsing Rust source input.
#[derive(Default, Debug)]
pub struct ParsedData {
    /// Structs defined in the source
    pub structs: Vec<RustStruct>,
    /// Enums defined in the source
    pub enums: Vec<RustEnum>,
    /// Type aliases defined in the source
    pub aliases: Vec<RustTypeAlias>,
}

impl ParsedData {
    /// Add the parsed data from `other` to `self`.
    pub fn add(&mut self, mut other: Self) {
        self.structs.append(&mut other.structs);
        self.enums.append(&mut other.enums);
        self.aliases.append(&mut other.aliases);
    }

    fn push_rust_thing(&mut self, rust_thing: RustThing) {
        match rust_thing {
            RustThing::Struct(s) => self.structs.push(s),
            RustThing::Enum(e) => self.enums.push(e),
            RustThing::Alias(a) => self.aliases.push(a),
        }
    }
}

/// Errors that can occur while parsing Rust source input.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ParseError {
    #[error("{0}")]
    SynError(#[from] syn::Error),
    #[error("failed to parse a rust type: {0}")]
    RustTypeParseError(#[from] RustTypeParseError),
    #[error("unsupported type encountered: {0}")]
    UnsupportedType(String),
    #[error("tuple structs with more than one field are currently unsupported")]
    ComplexTupleStruct,
    #[error("multiple unnamed associated types are not currently supported")]
    MultipleUnnamedAssociatedTypes,
    #[error("the serde tag attribute is not supported for non-algebraic enums: {enum_ident}")]
    SerdeTagNotAllowed { enum_ident: String },
    #[error("the serde content attribute is not supported for non-algebraic enums: {enum_ident}")]
    SerdeContentNotAllowed { enum_ident: String },
    #[error("serde tag attribute needs to be specified for algebraic enum {enum_ident}. e.g. #[serde(tag = \"type\", content = \"content\")]")]
    SerdeTagRequired { enum_ident: String },
    #[error("serde content attribute needs to be specified for algebraic enum {enum_ident}. e.g. #[serde(tag = \"type\", content = \"content\")]")]
    SerdeContentRequired { enum_ident: String },
}

/// Parse the given Rust source string into `ParsedData`.
pub fn parse(input: &str) -> Result<ParsedData, ParseError> {
    let mut parsed_data = ParsedData::default();

    // We will only produce output for files that contain the `#[typeshare]`
    // attribute, so this is a quick and easy performance win
    if !input.contains("typeshare") {
        return Ok(parsed_data);
    }

    // Parse and process the input, ensuring we parse only items marked with
    // `#[typeshare]
    let source = syn::parse_file(input)?;
    for item in &source.items {
        match item {
            syn::Item::Struct(s) if has_typeshare_annotation(&s.attrs) => {
                parsed_data.push_rust_thing(parse_struct(s)?);
            }
            syn::Item::Enum(e) if has_typeshare_annotation(&e.attrs) => {
                parsed_data.push_rust_thing(parse_enum(e)?);
            }
            syn::Item::Type(t) if has_typeshare_annotation(&t.attrs) => {
                parsed_data.aliases.push(parse_type_alias(t)?);
            }
            _ => {}
        }
    }

    Ok(parsed_data)
}

/// Allows parsing functions to return different things.
// TODO: this exists to allow for hacks in the code below, remove this
enum RustThing {
    Struct(RustStruct),
    Enum(RustEnum),
    Alias(RustTypeAlias),
}

/// Parses a struct into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than a struct, which is a
/// hack.
fn parse_struct(s: &ItemStruct) -> Result<RustThing, ParseError> {
    let serde_rename_all = serde_rename_all(&s.attrs);

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
    if let Some(ty) = get_serialized_as_type(&s.attrs) {
        return Ok(RustThing::Alias(RustTypeAlias {
            id: get_ident(Some(&s.ident), &s.attrs, &None),
            r#type: ty.parse()?,
            comments: parse_comment_attrs(&s.attrs),
            generic_types,
        }));
    }

    Ok(match &s.fields {
        // Structs
        Fields::Named(f) => {
            let fields = f
                .named
                .iter()
                .filter(|field| !is_skipped(&field.attrs))
                .map(|f| {
                    let ty = if let Some(ty) = get_field_type_override(&f.attrs) {
                        ty.parse()?
                    } else {
                        RustType::try_from(&f.ty)?
                    };

                    let has_default = serde_default(&f.attrs);
                    let decorators = get_field_decorators(&f.attrs);

                    Ok(RustField {
                        id: get_ident(f.ident.as_ref(), &f.attrs, &serde_rename_all),
                        ty,
                        comments: parse_comment_attrs(&f.attrs),
                        has_default,
                        decorators,
                    })
                })
                .collect::<Result<_, ParseError>>()?;

            RustThing::Struct(RustStruct {
                id: get_ident(Some(&s.ident), &s.attrs, &None),
                generic_types,
                fields,
                comments: parse_comment_attrs(&s.attrs),
                decorators: get_decorators(&s.attrs),
            })
        }
        // Tuple structs
        Fields::Unnamed(f) => {
            if f.unnamed.len() > 1 {
                return Err(ParseError::ComplexTupleStruct);
            }
            let f = &f.unnamed[0];

            let ty = if let Some(ty) = get_field_type_override(&f.attrs) {
                ty.parse()?
            } else {
                RustType::try_from(&f.ty)?
            };

            RustThing::Alias(RustTypeAlias {
                id: get_ident(Some(&s.ident), &s.attrs, &None),
                r#type: ty,
                comments: parse_comment_attrs(&s.attrs),
                generic_types,
            })
        }
        // Unit structs or `None`
        Fields::Unit => RustThing::Struct(RustStruct {
            id: get_ident(Some(&s.ident), &s.attrs, &None),
            generic_types,
            fields: vec![],
            comments: parse_comment_attrs(&s.attrs),
            decorators: get_decorators(&s.attrs),
        }),
    })
}

/// Parses an enum into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than an enum, which is a
/// hack.
fn parse_enum(e: &ItemEnum) -> Result<RustThing, ParseError> {
    let generic_types = e
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(type_param) => Some(type_param.ident.to_string()),
            _ => None,
        })
        .collect();

    let serde_rename_all = serde_rename_all(&e.attrs);

    // TODO: we shouldn't lie and return a type alias when parsing an enum. this
    // is a temporary hack
    if let Some(ty) = get_serialized_as_type(&e.attrs) {
        return Ok(RustThing::Alias(RustTypeAlias {
            id: get_ident(Some(&e.ident), &e.attrs, &None),
            r#type: ty.parse()?,
            comments: parse_comment_attrs(&e.attrs),
            generic_types,
        }));
    }

    let original_enum_ident = e.ident.to_string();

    // Grab the `#[serde(tag = "...", content = "...")]` values if they exist
    let maybe_tag_key = get_tag_key(&e.attrs);
    let maybe_content_key = get_content_key(&e.attrs);

    // Parse all of the enum's variants
    let variants = e
        .variants
        .iter()
        // Filter out variants we've been told to skip
        .filter(|v| !is_skipped(&v.attrs))
        .map(|v| parse_enum_variant(v, &serde_rename_all))
        .collect::<Result<Vec<_>, _>>()?;

    // Check if the enum references itself recursively in any of its variants
    let is_recursive = variants.iter().any(|v| match v {
        RustEnumVariant::Unit(_) => false,
        RustEnumVariant::Tuple { ty, .. } => ty.contains_type(&original_enum_ident),
        RustEnumVariant::AnonymousStruct { fields, .. } => fields
            .iter()
            .any(|f| f.ty.contains_type(&original_enum_ident)),
    });

    let shared = RustEnumShared {
        id: get_ident(Some(&e.ident), &e.attrs, &None),
        comments: parse_comment_attrs(&e.attrs),
        variants,
        decorators: get_decorators(&e.attrs),
        generic_types,
        is_recursive,
    };

    // Figure out if we're dealing with a unit enum or an algebraic enum
    if shared
        .variants
        .iter()
        .all(|v| matches!(v, RustEnumVariant::Unit(_)))
    {
        // All enum variants are unit-type

        if maybe_tag_key.is_some() {
            return Err(ParseError::SerdeTagNotAllowed {
                enum_ident: original_enum_ident,
            });
        }
        if maybe_content_key.is_some() {
            return Err(ParseError::SerdeContentNotAllowed {
                enum_ident: original_enum_ident,
            });
        }

        Ok(RustThing::Enum(RustEnum::Unit(shared)))
    } else {
        // At least one enum variant is either a tuple or an anonymous struct

        let tag_key = maybe_tag_key.ok_or_else(|| ParseError::SerdeTagRequired {
            enum_ident: original_enum_ident.clone(),
        })?;
        let content_key = maybe_content_key.ok_or_else(|| ParseError::SerdeContentRequired {
            enum_ident: original_enum_ident.clone(),
        })?;

        Ok(RustThing::Enum(RustEnum::Algebraic {
            tag_key,
            content_key,
            shared,
        }))
    }
}

/// Parse an enum variant.
fn parse_enum_variant(
    v: &syn::Variant,
    enum_serde_rename_all: &Option<String>,
) -> Result<RustEnumVariant, ParseError> {
    let shared = RustEnumVariantShared {
        id: get_ident(Some(&v.ident), &v.attrs, enum_serde_rename_all),
        comments: parse_comment_attrs(&v.attrs),
    };

    // Get the value of `#[serde(rename_all)]` for this specific variant rather
    // than the overall enum
    //
    // The value of the attribute for the enum overall does not apply to enum
    // variant fields.
    let variant_serde_rename_all = serde_rename_all(&v.attrs);

    match &v.fields {
        syn::Fields::Unit => Ok(RustEnumVariant::Unit(shared)),
        syn::Fields::Unnamed(associated_type) => {
            if associated_type.unnamed.len() > 1 {
                return Err(ParseError::MultipleUnnamedAssociatedTypes);
            }

            let first_field = associated_type.unnamed.first().unwrap();

            let ty = if let Some(ty) = get_field_type_override(&first_field.attrs) {
                ty.parse()?
            } else {
                RustType::try_from(&first_field.ty)?
            };

            Ok(RustEnumVariant::Tuple { ty, shared })
        }
        syn::Fields::Named(fields_named) => Ok(RustEnumVariant::AnonymousStruct {
            fields: fields_named
                .named
                .iter()
                .map(|f| {
                    let field_type = if let Some(ty) = get_field_type_override(&f.attrs) {
                        ty.parse()?
                    } else {
                        RustType::try_from(&f.ty)?
                    };

                    let has_default = serde_default(&f.attrs);
                    let decorators = get_field_decorators(&f.attrs);

                    Ok(RustField {
                        id: get_ident(f.ident.as_ref(), &f.attrs, &variant_serde_rename_all),
                        ty: field_type,
                        comments: parse_comment_attrs(&f.attrs),
                        has_default,
                        decorators,
                    })
                })
                .collect::<Result<Vec<_>, ParseError>>()?,
            shared,
        }),
    }
}

/// Parses a type alias into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
fn parse_type_alias(t: &ItemType) -> Result<RustTypeAlias, ParseError> {
    let ty = if let Some(ty) = get_serialized_as_type(&t.attrs) {
        ty.parse()?
    } else {
        RustType::try_from(t.ty.as_ref())?
    };

    let generic_types = t
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(type_param) => Some(type_param.ident.to_string()),
            _ => None,
        })
        .collect();

    Ok(RustTypeAlias {
        id: get_ident(Some(&t.ident), &t.attrs, &None),
        r#type: ty,
        comments: parse_comment_attrs(&t.attrs),
        generic_types,
    })
}

// Helpers

/// Parses any comment out of the given slice of attributes
fn parse_comment_attrs(attrs: &[Attribute]) -> Vec<String> {
    const DOC_ATTR: &str = "doc";
    attrs
        .iter()
        .map(Attribute::parse_meta)
        .filter_map(Result::ok)
        .filter_map(|attr| match attr {
            Meta::NameValue(name_value) => {
                if let Some(ident) = name_value.path.get_ident() {
                    if *ident == DOC_ATTR {
                        Some(name_value.lit)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .filter_map(literal_as_string)
        .map(|string| string.trim().into())
        .collect()
}

/// Checks the given attrs for `#[typeshare]`
fn has_typeshare_annotation(attrs: &[syn::Attribute]) -> bool {
    let typeshare_ident = Ident::new("typeshare", Span::call_site());
    for a in attrs {
        if let Some(segment) = a.path.segments.iter().next() {
            if segment.ident == typeshare_ident {
                return true;
            }
        }
    }

    false
}

fn get_ident(
    ident: Option<&proc_macro2::Ident>,
    attrs: &[syn::Attribute],
    rename_all: &Option<String>,
) -> Id {
    let original = ident.map_or("???".to_string(), |id| id.to_string().replace("r#", ""));

    let mut renamed = rename_all_to_case(original.clone(), rename_all);

    if let Some(s) = serde_rename(attrs) {
        renamed = s;
    }

    Id { original, renamed }
}

fn rename_all_to_case(original: String, case: &Option<String>) -> String {
    match case {
        None => original,
        Some(value) => match value.as_str() {
            "lowercase" => original.to_lowercase(),
            "UPPERCASE" => original.to_uppercase(),
            "PascalCase" => original.to_pascal_case(),
            "camelCase" => original.to_camel_case(),
            "snake_case" => original.to_snake_case(),
            "SCREAMING_SNAKE_CASE" => original.to_screaming_snake_case(),
            "kebab-case" => original.to_kebab_case(),
            "SCREAMING-KEBAB-CASE" => original.to_screaming_kebab_case(),
            _ => original,
        },
    }
}

fn literal_as_string(lit: syn::Lit) -> Option<String> {
    match lit {
        syn::Lit::Str(str) => Some(str.value()),
        _ => None,
    }
}

fn get_typeshare_name_value_meta_items<'a>(
    attrs: &'a [syn::Attribute],
    name: &'a str,
) -> impl Iterator<Item = syn::Lit> + 'a {
    attrs.iter().flat_map(move |attr| {
        get_typeshare_meta_items(attr)
            .iter()
            .filter_map(|arg| match arg {
                NestedMeta::Meta(Meta::NameValue(name_value)) => {
                    if let Some(ident) = name_value.path.get_ident() {
                        if *ident == name {
                            Some(name_value.lit.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    })
}

fn get_serde_name_value_meta_items<'a>(
    attrs: &'a [syn::Attribute],
    name: &'a str,
) -> impl Iterator<Item = syn::Lit> + 'a {
    attrs.iter().flat_map(move |attr| {
        get_serde_meta_items(attr)
            .iter()
            .filter_map(|arg| match arg {
                NestedMeta::Meta(Meta::NameValue(name_value)) => {
                    if let Some(ident) = name_value.path.get_ident() {
                        if *ident == name {
                            Some(name_value.lit.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    })
}

fn get_serialized_as_type(attrs: &[syn::Attribute]) -> Option<String> {
    get_typeshare_name_value_meta_items(attrs, "serialized_as")
        .next()
        .and_then(literal_as_string)
}

fn get_field_type_override(attrs: &[syn::Attribute]) -> Option<String> {
    get_typeshare_name_value_meta_items(attrs, "serialized_as")
        .next()
        .and_then(literal_as_string)
}

/// Checks the struct or enum for decorators like `#[typeshare(typescript(readonly)]`
/// Takes a slice of `syn::Attribute`, returns a `HashMap<language, HashSet<decoration_words>>`, where `language` and `decoration_words` are `String`s
fn get_field_decorators(attrs: &[syn::Attribute]) -> HashMap<String, HashSet<String>> {
    let languages: HashSet<String> = ["typescript", "kotlin", "go", "swift"]
        .iter()
        .map(|l| l.to_string())
        .collect();

    attrs
        .iter()
        .flat_map(get_typeshare_meta_items)
        .flat_map(|meta| {
            if let NestedMeta::Meta(Meta::List(list)) = meta {
                Some(list)
            } else {
                None
            }
        })
        .flat_map(|list| match list.path.get_ident() {
            Some(ident) if languages.contains(&ident.to_string()) => {
                Some((ident.to_string(), list.nested))
            }
            _ => None,
        })
        .map(|(language, list)| {
            (
                language,
                list.iter()
                    .flat_map(|nested| match nested {
                        NestedMeta::Meta(Meta::Path(path)) => path.get_ident(),
                        _ => None,
                    })
                    .map(|ident| ident.to_string())
                    .collect::<HashSet<String>>(),
            )
        })
        .fold(HashMap::new(), |mut acc, (language, decorators)| {
            acc.entry(language).or_default().extend(decorators);
            acc
        })
}

/// Checks the struct or enum for decorators like `#[typeshare(swift = "Codable, Equatable")]`
/// Takes a slice of `syn::Attribute`, returns a `HashMap<language, Vec<decoration_words>>`, where `language` and `decoration_words` are `String`s
fn get_decorators(attrs: &[syn::Attribute]) -> HashMap<String, Vec<String>> {
    // The resulting HashMap, Key is the language, and the value is a vector of decorators words that will be put onto structures
    let mut out: HashMap<String, Vec<String>> = HashMap::new();

    for value in get_typeshare_name_value_meta_items(attrs, "swift").filter_map(literal_as_string) {
        let decorators: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();

        // lastly, get the entry in the hashmap output and extend the value, or insert what we have already found
        let decs = out.entry("swift".to_string()).or_insert_with(Vec::new);
        decs.extend(decorators);
        // Sorting so all the added decorators will be after the normal ([`String`], `Codable`) in alphabetical order
        decs.sort_unstable();
        decs.dedup(); //removing any duplicates just in case
    }

    //return our hashmap mapping of language -> Vec<decorators>
    out
}

fn get_tag_key(attrs: &[syn::Attribute]) -> Option<String> {
    get_serde_name_value_meta_items(attrs, "tag")
        .next()
        .and_then(literal_as_string)
}

fn get_content_key(attrs: &[syn::Attribute]) -> Option<String> {
    get_serde_name_value_meta_items(attrs, "content")
        .next()
        .and_then(literal_as_string)
}

fn serde_rename(attrs: &[syn::Attribute]) -> Option<String> {
    get_serde_name_value_meta_items(attrs, "rename")
        .next()
        .and_then(literal_as_string)
}

fn serde_rename_all(attrs: &[syn::Attribute]) -> Option<String> {
    get_serde_name_value_meta_items(attrs, "rename_all")
        .next()
        .and_then(literal_as_string)
}

fn serde_default(attrs: &[syn::Attribute]) -> bool {
    let default = Ident::new("default", Span::call_site());

    attrs.iter().any(|attr| {
        get_serde_meta_items(attr).iter().any(|arg| match arg {
            NestedMeta::Meta(Meta::Path(path)) => {
                if let Some(ident) = path.get_ident() {
                    *ident == default
                } else {
                    false
                }
            }
            _ => false,
        })
    })
}

// TODO: for now, this is a workaround until we can integrate serde_derive_internal
// into our parser.
/// Returns all arguments passed into `#[serde(...)]` attributes
pub fn get_serde_meta_items(attr: &syn::Attribute) -> Vec<NestedMeta> {
    if attr.path.get_ident().is_none() || *attr.path.get_ident().unwrap() != SERDE {
        return Vec::default();
    }

    match attr.parse_meta() {
        Ok(Meta::List(meta)) => meta.nested.into_iter().collect(),
        _ => Vec::new(),
    }
}

/// Returns all arguments passed into `#[typeshare(...)]` attributes
pub fn get_typeshare_meta_items(attr: &syn::Attribute) -> Vec<NestedMeta> {
    if attr.path.get_ident().is_none() || *attr.path.get_ident().unwrap() != TYPESHARE {
        return Vec::default();
    }

    match attr.parse_meta() {
        Ok(Meta::List(meta)) => meta.nested.into_iter().collect(),
        _ => Vec::new(),
    }
}

// `#[typeshare(skip)]` or `#[serde(skip)]`
fn is_skipped(attrs: &[syn::Attribute]) -> bool {
    let skip = Ident::new("skip", Span::call_site());
    attrs.iter().any(|attr| {
        get_serde_meta_items(attr)
            .into_iter()
            .chain(get_typeshare_meta_items(attr).into_iter())
            .any(|arg| match arg {
                NestedMeta::Meta(Meta::Path(path)) => {
                    if let Some(ident) = path.get_ident() {
                        *ident == skip
                    } else {
                        false
                    }
                }
                _ => false,
            })
    })
}

#[test]
fn test_rename_all_to_case() {
    let test_word = "test_case";

    let tests = [
        ("lowercase", "test_case"),
        ("UPPERCASE", "TEST_CASE"),
        ("PascalCase", "TestCase"),
        ("camelCase", "testCase"),
        ("snake_case", "test_case"),
        ("SCREAMING_SNAKE_CASE", "TEST_CASE"),
        ("kebab-case", "test-case"),
        ("SCREAMING-KEBAB-CASE", "TEST-CASE"),
        ("invalid case", "test_case"),
    ];

    for test in tests {
        assert_eq!(
            rename_all_to_case(test_word.to_string(), &Some(test.0.to_string())),
            test.1
        );
    }
}

/// Removes `-` characters from identifiers
pub(crate) fn remove_dash_from_identifier(name: &str) -> String {
    // Dashes are not valid in identifiers, so we map them to underscores
    name.replace('-', "_")
}
