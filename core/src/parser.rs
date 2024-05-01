use crate::{
    language::SupportedLanguage,
    rename::RenameExt,
    rust_types::{
        FieldDecorator, Id, RustEnum, RustEnumShared, RustEnumVariant, RustEnumVariantShared,
        RustField, RustItem, RustStruct, RustType, RustTypeAlias, RustTypeParseError,
    },
    visitors::{ImportedType, TypeShareVisitor},
};
use proc_macro2::Ident;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    convert::TryFrom,
};
use syn::{
    ext::IdentExt, parse::ParseBuffer, punctuated::Punctuated, visit::Visit, Attribute, Expr,
    ExprLit, Fields, GenericParam, ItemEnum, ItemStruct, ItemType, LitStr, Meta, MetaList,
    MetaNameValue, Token,
};
use thiserror::Error;

const TYPESHARE: &str = "typeshare";
const SERDE: &str = "serde";

/// Errors that can occur while parsing Rust source input.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum ParseError {
    #[error("{0}")]
    SynError(#[from] syn::Error),
    #[error("failed to parse a rust type: {0}")]
    RustTypeParseError(#[from] RustTypeParseError),
    #[error("unsupported language encountered: {0}")]
    UnsupportedLanguage(String),
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
    #[error("the serde flatten attribute is not currently supported")]
    SerdeFlattenNotAllowed,
}

/// Error with it's related data.
#[derive(Debug)]
pub struct ErrorInfo {
    /// The crate where this error occured.
    pub crate_name: String,
    /// The file name being parsed.
    pub file_name: String,
    /// The parse error.
    pub error: ParseError,
}

/// The results of parsing Rust source input.
#[derive(Default, Debug)]
pub struct ParsedData {
    /// Structs defined in the source
    pub structs: Vec<RustStruct>,
    /// Enums defined in the source
    pub enums: Vec<RustEnum>,
    /// Type aliases defined in the source
    pub aliases: Vec<RustTypeAlias>,
    /// Imports used by this file
    pub import_types: Vec<ImportedType>,
    /// Crate this belongs to.
    pub crate_name: String,
    /// File name to write to for generated type.
    pub file_name: String,
    /// All type names
    pub type_names: Vec<String>,
    /// Failures during parsing.
    pub errors: Vec<ErrorInfo>,
}

pub struct ParsedModule {
    pub module: HashMap<String, Vec<ParsedData>>,
}

impl ParsedData {
    pub fn new(crate_name: String, file_name: String) -> Self {
        Self {
            crate_name,
            file_name,
            ..Default::default()
        }
    }

    /// Add the parsed data from `other` to `self`.
    pub fn add(&mut self, mut other: Self) {
        self.structs.append(&mut other.structs);
        self.enums.append(&mut other.enums);
        self.aliases.append(&mut other.aliases);
        self.import_types.append(&mut other.import_types);
        self.type_names.append(&mut other.type_names);
        self.errors.append(&mut other.errors);
    }

    pub(crate) fn push(&mut self, rust_thing: RustItem) {
        match rust_thing {
            RustItem::Struct(s) => {
                self.type_names.push(s.id.renamed.clone());
                self.structs.push(s);
            }
            RustItem::Enum(e) => {
                self.type_names.push(e.shared().id.renamed.clone());
                self.enums.push(e);
            }
            RustItem::Alias(a) => {
                self.type_names.push(a.id.renamed.clone());
                self.aliases.push(a);
            }
        }
    }
}

/// Parse the given Rust source string into `ParsedData`.
pub fn parse(
    input: &str,
    crate_name: String,
    file_name: String,
) -> Result<Option<ParsedData>, ParseError> {
    // We will only produce output for files that contain the `#[typeshare]`
    // attribute, so this is a quick and easy performance win
    if !input.contains(TYPESHARE) {
        return Ok(None);
    }

    // let mut parsed_data = ParsedData::new(crate_name.clone(), file_name);
    // Parse and process the input, ensuring we parse only items marked with
    // `#[typeshare]`
    let source = syn::parse_file(input)?;

    let mut import_visitor = TypeShareVisitor::new(crate_name, file_name);
    import_visitor.visit_file(&source);

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
/// Takes a slice of `syn::Attribute`, returns a `HashMap<language, Vec<decoration_words>>`, where `language` is `SupportedLanguage` and `decoration_words` is `String`
fn get_decorators(attrs: &[syn::Attribute]) -> HashMap<SupportedLanguage, Vec<String>> {
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
