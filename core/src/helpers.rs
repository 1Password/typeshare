use std::collections::{BTreeSet, HashMap, HashSet};

use proc_macro2::Ident;
use syn::{
    ext::IdentExt, parse::ParseBuffer, punctuated::Punctuated, Attribute, Expr, ExprLit, LitStr,
    Meta, MetaList, MetaNameValue, Token,
};

use crate::{
    language::SupportedLanguage,
    rename::RenameExt,
    rust_types::{FieldDecorator, Id},
};

const SERDE: &str = "serde";
const TYPESHARE: &str = "typeshare";

/// Checks the given attrs for `#[typeshare]`
pub(crate) fn has_typeshare_annotation(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .flat_map(|attr| attr.path().segments.clone())
        .any(|segment| segment.ident == TYPESHARE)
}

pub(crate) fn serde_rename_all(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "rename_all", SERDE).next()
}

pub(crate) fn get_serialized_as_type(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "serialized_as", TYPESHARE).next()
}

pub(crate) fn get_field_type_override(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "serialized_as", TYPESHARE).next()
}

pub(crate) fn get_name_value_meta_items<'a>(
    attrs: &'a [syn::Attribute],
    name: &'a str,
    ident: &'static str,
) -> impl Iterator<Item = String> + 'a {
    attrs.iter().flat_map(move |attr| {
        get_meta_items(attr, ident)
            .iter()
            .filter_map(|arg| match arg {
                Meta::NameValue(name_value) if name_value.path.is_ident(name) => {
                    expr_to_string(&name_value.value)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    })
}

/// Returns all arguments passed into `#[{ident}(...)]` where `{ident}` can be `serde` or `typeshare` attributes
pub(crate) fn get_meta_items(attr: &syn::Attribute, ident: &str) -> Vec<Meta> {
    if attr.path().is_ident(ident) {
        attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .iter()
            .flat_map(|meta| meta.iter())
            .cloned()
            .collect()
    } else {
        Vec::default()
    }
}

pub(crate) fn get_ident(
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

pub(crate) fn rename_all_to_case(original: String, case: &Option<String>) -> String {
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

pub(crate) fn serde_rename(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "rename", SERDE).next()
}

/// Parses any comment out of the given slice of attributes
pub(crate) fn parse_comment_attrs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .map(|attr| attr.meta.clone())
        .filter_map(|meta| match meta {
            Meta::NameValue(name_value) if name_value.path.is_ident("doc") => {
                expr_to_string(&name_value.value)
            }
            _ => None,
        })
        .collect()
}

// `#[typeshare(skip)]` or `#[serde(skip)]`
pub(crate) fn is_skipped(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        get_meta_items(attr, SERDE)
            .into_iter()
            .chain(get_meta_items(attr, TYPESHARE))
            .any(|arg| matches!(arg, Meta::Path(path) if path.is_ident("skip")))
    })
}

fn serde_attr(attrs: &[syn::Attribute], ident: &str) -> bool {
    attrs.iter().any(|attr| {
        get_meta_items(attr, SERDE)
            .iter()
            .any(|arg| matches!(arg, Meta::Path(path) if path.is_ident(ident)))
    })
}

pub(crate) fn serde_default(attrs: &[syn::Attribute]) -> bool {
    serde_attr(attrs, "default")
}

pub(crate) fn serde_flatten(attrs: &[syn::Attribute]) -> bool {
    serde_attr(attrs, "flatten")
}

/// Checks the struct or enum for decorators like `#[typeshare(typescript(readonly)]`
/// Takes a slice of `syn::Attribute`, returns a `HashMap<language, BTreeSet<decorator>>`, where `language` is `SupportedLanguage`
/// and `decorator` is `FieldDecorator`. Field decorators are ordered in a `BTreeSet` for consistent code generation.
pub(crate) fn get_field_decorators(
    attrs: &[Attribute],
) -> HashMap<SupportedLanguage, BTreeSet<FieldDecorator>> {
    let languages: HashSet<SupportedLanguage> = SupportedLanguage::all_languages().collect();

    attrs
        .iter()
        .flat_map(|attr| get_meta_items(attr, TYPESHARE))
        .flat_map(|meta| {
            if let Meta::List(list) = meta {
                Some(list)
            } else {
                None
            }
        })
        .flat_map(|list: MetaList| match list.path.get_ident() {
            Some(ident) if languages.contains(&ident.try_into().unwrap()) => {
                Some((ident.try_into().unwrap(), list))
            }
            _ => None,
        })
        .map(|(language, list): (SupportedLanguage, MetaList)| {
            (
                language,
                list.parse_args_with(|input: &ParseBuffer| {
                    let mut res: Vec<Meta> = vec![];

                    loop {
                        if input.is_empty() {
                            break;
                        }

                        let ident = input.call(Ident::parse_any)?;

                        // Parse `readonly` or any other single ident optionally followed by a comma
                        if input.peek(Token![,]) || input.is_empty() {
                            input.parse::<Token![,]>().unwrap_or_default();
                            res.push(Meta::Path(ident.into()));
                            continue;
                        }

                        if input.is_empty() {
                            break;
                        }

                        // Parse `= "any | undefined"` or any other eq sign followed by a string literal

                        let eq_token = input.parse::<Token![=]>()?;

                        let value: LitStr = input.parse()?;
                        res.push(Meta::NameValue(MetaNameValue {
                            path: ident.into(),
                            eq_token,
                            value: Expr::Lit(ExprLit {
                                attrs: Vec::new(),
                                lit: value.into(),
                            }),
                        }));

                        if input.is_empty() {
                            break;
                        }

                        input.parse::<Token![,]>()?;
                    }
                    Ok(res)
                })
                .iter()
                .flatten()
                .filter_map(|nested| match nested {
                    Meta::Path(path) if path.segments.len() == 1 => {
                        Some(FieldDecorator::Word(path.get_ident()?.to_string()))
                    }
                    Meta::NameValue(name_value) => Some(FieldDecorator::NameValue(
                        name_value.path.get_ident()?.to_string(),
                        expr_to_string(&name_value.value)?,
                    )),
                    // TODO: this should throw a visible error since it suggests a malformed
                    //       attribute.
                    _ => None,
                })
                .collect::<Vec<FieldDecorator>>(),
            )
        })
        .fold(HashMap::new(), |mut acc, (language, decorators)| {
            acc.entry(language).or_default().extend(decorators);
            acc
        })
}

fn expr_to_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(expr_lit) => literal_to_string(&expr_lit.lit),
        _ => None,
    }
}

fn literal_to_string(lit: &syn::Lit) -> Option<String> {
    match lit {
        syn::Lit::Str(str) => Some(str.value().trim().to_string()),
        _ => None,
    }
}

/// Checks the struct or enum for decorators like `#[typeshare(swift = "Codable, Equatable")]`
/// Takes a slice of `syn::Attribute`, returns a `HashMap<language, Vec<decoration_words>>`, where `language` is `SupportedLanguage` and `decoration_words` is `String`
pub(crate) fn get_decorators(attrs: &[syn::Attribute]) -> HashMap<SupportedLanguage, Vec<String>> {
    // The resulting HashMap, Key is the language, and the value is a vector of decorators words that will be put onto structures
    let mut out: HashMap<SupportedLanguage, Vec<String>> = HashMap::new();

    for value in get_name_value_meta_items(attrs, "swift", TYPESHARE) {
        let decorators: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();

        // lastly, get the entry in the hashmap output and extend the value, or insert what we have already found
        let decs = out.entry(SupportedLanguage::Swift).or_default();
        decs.extend(decorators);
        // Sorting so all the added decorators will be after the normal ([`String`], `Codable`) in alphabetical order
        decs.sort_unstable();
        decs.dedup(); //removing any duplicates just in case
    }

    //return our hashmap mapping of language -> Vec<decorators>
    out
}

pub(crate) fn get_tag_key(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "tag", SERDE).next()
}

pub(crate) fn get_content_key(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "content", SERDE).next()
}

/// Removes `-` characters from identifiers
pub(crate) fn remove_dash_from_identifier(name: &str) -> String {
    // Dashes are not valid in identifiers, so we map them to underscores
    name.replace('-', "_")
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
