//! Source file parsing.
use anyhow::Context;
use heck::ToPascalCase;
use ignore::WalkBuilder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{hash_map::Entry, HashMap},
    ops::Not,
    path::PathBuf,
};
use syn::{punctuated::Punctuated, Fields, GenericParam, ItemEnum, ItemStruct};

use typeshare_model::{
    parsed_data::{RustEnumShared, RustItem},
    prelude::*,
};

/// Input data for parsing each source file.
pub struct ParserInput {
    /// Rust source file path.
    file_path: PathBuf,
    /// File name source from crate for output.
    file_name: String,
    /// The crate name the source file belongs to.
    crate_name: CrateName,
}

// TODO: put this in the language trait
pub struct LangConfig {
    extension: &'static str,
    pascal: bool,
}

/// Walk the source folder and collect all parser inputs.
pub fn parser_inputs(
    walker_builder: WalkBuilder,
    language_type: &LangConfig,
    multi_file: bool,
) -> Vec<ParserInput> {
    walker_builder
        .build()
        .filter_map(Result::ok)
        .filter(|dir_entry| !dir_entry.path().is_dir())
        .filter_map(|dir_entry| {
            let crate_name = if multi_file {
                CrateName::find_crate_name(dir_entry.path())?
            } else {
                SINGLE_FILE_CRATE_NAME
            };
            let file_path = dir_entry.path().to_path_buf();
            let file_name = output_file_name(language_type, &crate_name);
            Some(ParserInput {
                file_path,
                file_name,
                crate_name,
            })
        })
        .collect()
}

/// The output file name to write to.
fn output_file_name(language_type: &LangConfig, crate_name: &CrateName) -> String {
    // TODO: improve all of this with the language trait
    let extension = language_type.extension;

    let snake_case = || format!("{crate_name}.{extension}");
    let pascal_case = || format!("{}.{extension}", crate_name.to_string().to_pascal_case());

    match language_type.pascal {
        true => pascal_case(),
        false => snake_case(),
    }
}

/// Collect all the typeshared types into a mapping of crate names to typeshared types. This
/// mapping is used to lookup and generated import statements for generated files.
pub fn all_types(file_mappings: &HashMap<CrateName, ParsedData>) -> CrateTypes {
    file_mappings
        .iter()
        .map(|(crate_name, parsed_data)| (crate_name, parsed_data.type_names.clone()))
        .fold(
            HashMap::new(),
            |mut import_map: CrateTypes, (crate_name, type_names)| {
                let type_names = type_names.into_iter().map(TypeName);

                match import_map.entry(crate_name.clone()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().extend(type_names);
                    }
                    Entry::Vacant(e) => {
                        e.insert(type_names.collect());
                    }
                }
                import_map
            },
        )
}

/// Collect all the parsed sources into a mapping of crate name to parsed data.
pub fn parse_input(
    inputs: Vec<ParserInput>,
    ignored_types: &[&str],
    multi_file: bool,
) -> anyhow::Result<HashMap<CrateName, ParsedData>> {
    inputs
        .into_par_iter()
        .try_fold(
            HashMap::new,
            |mut results: HashMap<CrateName, ParsedData>,
             ParserInput {
                 file_path,
                 file_name,
                 crate_name,
             }| {
                match std::fs::read_to_string(&file_path)
                    .context("Failed to read input")
                    .and_then(|data| {
                        parse(
                            &data,
                            crate_name.clone(),
                            file_name.clone(),
                            file_path,
                            ignored_types,
                            multi_file,
                        )
                        .context("Failed to parse")
                    })
                    .map(|parsed_data| {
                        parsed_data.and_then(|parsed_data| {
                            is_parsed_data_empty(&parsed_data)
                                .not()
                                .then_some((crate_name, parsed_data))
                        })
                    })? {
                    Some((crate_name, parsed_data)) => {
                        match results.entry(crate_name) {
                            Entry::Occupied(mut entry) => {
                                entry.get_mut().merge(parsed_data);
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(parsed_data);
                            }
                        }
                        Ok::<_, anyhow::Error>(results)
                    }
                    None => Ok(results),
                }
            },
        )
        .try_reduce(HashMap::new, |mut file_maps, mapping| {
            for (crate_name, parsed_data) in mapping {
                match file_maps.entry(crate_name) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().merge(parsed_data);
                    }
                    Entry::Vacant(e) => {
                        e.insert(parsed_data);
                    }
                }
            }
            Ok(file_maps)
        })
}

/// Check if we have not parsed any relavent typehsared types.
fn is_parsed_data_empty(parsed_data: &ParsedData) -> bool {
    parsed_data.enums.is_empty() && parsed_data.aliases.is_empty() && parsed_data.structs.is_empty()
}

/// Parse the given Rust source string into `ParsedData`.
pub fn parse(
    source_code: &str,
    crate_name: CrateName,
    file_name: String,
    file_path: PathBuf,
    ignored_types: &[&str],
    mult_file: bool,
) -> Result<Option<ParsedData>, ParseError> {
    // We will only produce output for files that contain the `#[typeshare]`
    // attribute, so this is a quick and easy performance win
    if !source_code.contains("#[typeshare") {
        return Ok(None);
    }

    // Parse and process the input, ensuring we parse only items marked with
    // `#[typeshare]`
    let mut import_visitor =
        TypeShareVisitor::new(crate_name, file_name, file_path, ignored_types, mult_file);
    import_visitor.visit_file(&syn::parse_file(source_code)?);

    Ok(Some(import_visitor.parsed_data()))
}

/// Parses a struct into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than a struct, which is a
/// hack.
pub(crate) fn parse_struct(s: &ItemStruct) -> Result<RustItem, ParseError> {
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
        return Ok(RustItem::Alias(RustTypeAlias {
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

                    if serde_flatten(&f.attrs) {
                        return Err(ParseError::SerdeFlattenNotAllowed);
                    }

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

            RustItem::Struct(RustStruct {
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

            RustItem::Alias(RustTypeAlias {
                id: get_ident(Some(&s.ident), &s.attrs, &None),
                r#type: ty,
                comments: parse_comment_attrs(&s.attrs),
                generic_types,
            })
        }
        // Unit structs or `None`
        Fields::Unit => RustItem::Struct(RustStruct {
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
pub(crate) fn parse_enum(e: &ItemEnum) -> Result<RustItem, ParseError> {
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
        return Ok(RustItem::Alias(RustTypeAlias {
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

        Ok(RustItem::Enum(RustEnum::Unit(shared)))
    } else {
        // At least one enum variant is either a tuple or an anonymous struct

        let tag_key = maybe_tag_key.ok_or_else(|| ParseError::SerdeTagRequired {
            enum_ident: original_enum_ident.clone(),
        })?;
        let content_key = maybe_content_key.ok_or_else(|| ParseError::SerdeContentRequired {
            enum_ident: original_enum_ident.clone(),
        })?;

        Ok(RustItem::Enum(RustEnum::Algebraic {
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
                .filter(|f| !is_skipped(&f.attrs))
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
pub(crate) fn parse_type_alias(t: &ItemType) -> Result<RustItem, ParseError> {
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

    Ok(RustItem::Alias(RustTypeAlias {
        id: get_ident(Some(&t.ident), &t.attrs, &None),
        r#type: ty,
        comments: parse_comment_attrs(&t.attrs),
        generic_types,
    }))
}

// Helpers

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
fn get_meta_items(attr: &syn::Attribute, ident: &str) -> Vec<Meta> {
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

fn serde_rename(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "rename", SERDE).next()
}

/// Parses any comment out of the given slice of attributes
fn parse_comment_attrs(attrs: &[Attribute]) -> Vec<String> {
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
fn is_skipped(attrs: &[syn::Attribute]) -> bool {
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

fn serde_default(attrs: &[syn::Attribute]) -> bool {
    serde_attr(attrs, "default")
}

fn serde_flatten(attrs: &[syn::Attribute]) -> bool {
    serde_attr(attrs, "flatten")
}

/// Checks the struct or enum for decorators like `#[typeshare(typescript(readonly)]`
/// Takes a slice of `syn::Attribute`, returns a `HashMap<language, BTreeSet<decorator>>`, where `language` is `SupportedLanguage`
/// and `decorator` is `FieldDecorator`. Field decorators are ordered in a `BTreeSet` for consistent code generation.
fn get_field_decorators(
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
/// Takes a slice of `syn::Attribute`, returns a [`DecoratorMap`].
fn get_decorators(attrs: &[syn::Attribute]) -> DecoratorMap {
    let mut decorator_map: DecoratorMap = DecoratorMap::new();

    for decorator_kind in [
        DecoratorKind::Swift,
        DecoratorKind::SwiftGenericConstraints,
        DecoratorKind::Kotlin,
    ] {
        for value in get_name_value_meta_items(attrs, decorator_kind.as_str(), TYPESHARE) {
            decorator_map
                .entry(decorator_kind)
                .or_default()
                .extend(value.split(',').map(|s| s.trim().to_string()));
        }
    }

    decorator_map
}

fn get_tag_key(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "tag", SERDE).next()
}

fn get_content_key(attrs: &[syn::Attribute]) -> Option<String> {
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
