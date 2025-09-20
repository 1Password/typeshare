use crate::{
    context::{ParseContext, ParseFileContext},
    error::{ParseError, ParseErrorWithSpan, WithSpan as _},
    language::{CrateName, SupportedLanguage},
    rename::RenameExt,
    rust_types::{
        DecoratorMap, FieldDecorator, Id, RustConst, RustConstExpr, RustEnum, RustEnumShared,
        RustEnumVariant, RustEnumVariantShared, RustField, RustItem, RustStruct, RustType,
        RustTypeAlias, SpecialRustType,
    },
    target_os_check::accept_target_os,
    visitors::{ImportedType, TypeShareVisitor},
};
use itertools::Either;
use log::debug;
use proc_macro2::Ident;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    convert::TryFrom,
    ops::AddAssign,
};
use syn::{
    ext::IdentExt, parse::ParseBuffer, punctuated::Punctuated, spanned::Spanned as _, visit::Visit,
    Attribute, Expr, ExprLit, Fields, GenericParam, ItemConst, ItemEnum, ItemStruct, ItemType, Lit,
    LitStr, Meta, MetaList, MetaNameValue, Token,
};

const TYPESHARE: &str = "typeshare";
const SERDE: &str = "serde";

/// Supported typeshare type level decorator attributes.
#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
pub enum DecoratorKind {
    /// The typeshare attribute for swift type constraints "swift"
    Swift,
    /// The typeshare attribute for swift generic constraints "swiftGenericConstraints"
    SwiftGenericConstraints,
    /// The typeshare attribute for kotlin "kotlin"
    Kotlin,
}

impl DecoratorKind {
    /// This decorator as a str.
    fn as_str(&self) -> &str {
        match self {
            DecoratorKind::Swift => "swift",
            DecoratorKind::SwiftGenericConstraints => "swiftGenericConstraints",
            DecoratorKind::Kotlin => "kotlin",
        }
    }
}

/// Error with it's related data.
#[derive(Debug)]
pub struct ErrorInfo {
    /// The file name being parsed.
    pub file_name: String,
    /// The parse error.
    pub error: String,
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
    /// Constant variables defined in the source
    pub consts: Vec<RustConst>,
    /// Imports used by this file
    pub import_types: HashSet<ImportedType>,
    /// Crate this belongs to.
    pub crate_name: CrateName,
    /// File name to write to for generated type.
    pub file_name: String,
    /// All type names
    pub type_names: HashSet<String>,
    /// Failures during parsing.
    pub errors: Vec<ErrorInfo>,
    /// Using multi file support.
    pub multi_file: bool,
}

// The better abstraction here is Semigroup Monoid but such
// traits don't exist in Rust. I'd rather have infix <>.
impl AddAssign<ParsedData> for ParsedData {
    fn add_assign(&mut self, mut rhs: ParsedData) {
        self.structs.append(&mut rhs.structs);
        self.enums.append(&mut rhs.enums);
        self.aliases.append(&mut rhs.aliases);
        self.consts.append(&mut rhs.consts);
        self.import_types.extend(rhs.import_types);
        self.type_names.extend(rhs.type_names);
        self.errors.append(&mut rhs.errors);

        self.file_name = rhs.file_name;
        self.crate_name = rhs.crate_name;
        self.multi_file = rhs.multi_file;
    }
}

impl ParsedData {
    /// Create a new parsed data.
    pub fn new(crate_name: CrateName, file_name: String, multi_file: bool) -> Self {
        Self {
            crate_name,
            file_name,
            multi_file,
            ..Default::default()
        }
    }

    pub(crate) fn push(&mut self, rust_thing: RustItem) {
        match rust_thing {
            RustItem::Struct(s) => {
                self.type_names.insert(s.id.renamed.clone());
                self.structs.push(s);
            }
            RustItem::Enum(e) => {
                self.type_names.insert(e.shared().id.renamed.clone());
                self.enums.push(e);
            }
            RustItem::Alias(a) => {
                self.type_names.insert(a.id.renamed.clone());
                self.aliases.push(a);
            }
            RustItem::Const(c) => {
                self.type_names.insert(c.id.renamed.clone());
                self.consts.push(c);
            }
        }
    }

    /// If this file was skipped by the visitor.
    pub fn is_empty(&self) -> bool {
        self.structs.is_empty()
            && self.enums.is_empty()
            && self.aliases.is_empty()
            && self.consts.is_empty()
            && self.errors.is_empty()
    }
}

/// Parse the given Rust source string into `ParsedData`.
pub fn parse(
    parse_context: &ParseContext,
    parse_file_context: ParseFileContext,
) -> Result<Option<ParsedData>, ParseErrorWithSpan> {
    // We will only produce output for files that contain the `#[typeshare]`
    // attribute, so this is a quick and easy performance win
    if !parse_file_context.source_code.contains("#[typeshare") {
        return Ok(None);
    }

    let ParseFileContext {
        source_code,
        crate_name,
        file_name,
        file_path,
    } = parse_file_context;

    debug!("parsing {file_path:?}");
    // Parse and process the input, ensuring we parse only items marked with
    // `#[typeshare]`
    let mut import_visitor = TypeShareVisitor::new(parse_context, crate_name, file_name, file_path);
    import_visitor.visit_file(&syn::parse_file(&source_code).map_err(|err| {
        let span = err.span();
        ParseError::from(err).with_span(span)
    })?);

    Ok(import_visitor.parsed_data())
}

/// Parses a struct into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than a struct, which is a
/// hack.
pub(crate) fn parse_struct(
    s: &ItemStruct,
    target_os: &[String],
) -> Result<RustItem, ParseErrorWithSpan> {
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
            decorators: get_decorators(&s.attrs),
            is_redacted: is_redacted(&s.attrs),
        }));
    }

    Ok(match &s.fields {
        // Structs
        Fields::Named(f) => {
            let fields = f
                .named
                .iter()
                .inspect(|field| debug!("\t\tChecking field {:?}", field.ident))
                .filter(|field| !is_skipped(&field.attrs, target_os))
                .inspect(|field| debug!("\t\tAccepted field {:?}", field.ident))
                .map(|f| {
                    let ty = if let Some(ty) = get_field_type_override(&f.attrs) {
                        ty.parse()?
                    } else {
                        RustType::try_from(&f.ty)?
                    };

                    if serde_flatten(&f.attrs) {
                        return Err(ParseError::SerdeFlattenNotAllowed.with_span(f.span()));
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
                .collect::<Result<_, ParseErrorWithSpan>>()?;

            RustItem::Struct(RustStruct {
                id: get_ident(Some(&s.ident), &s.attrs, &None),
                generic_types,
                fields,
                comments: parse_comment_attrs(&s.attrs),
                decorators: get_decorators(&s.attrs),
                is_redacted: is_redacted(&s.attrs),
            })
        }
        // Tuple structs
        Fields::Unnamed(f) => {
            if f.unnamed.len() > 1 {
                return Err(ParseError::ComplexTupleStruct.with_span(f.span()));
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
                decorators: get_decorators(&s.attrs),
                is_redacted: is_redacted(&s.attrs),
            })
        }
        // Unit structs or `None`
        Fields::Unit => RustItem::Struct(RustStruct {
            id: get_ident(Some(&s.ident), &s.attrs, &None),
            generic_types,
            fields: vec![],
            comments: parse_comment_attrs(&s.attrs),
            decorators: get_decorators(&s.attrs),
            is_redacted: is_redacted(&s.attrs),
        }),
    })
}

/// Parses an enum into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
///
/// This function can currently return something other than an enum, which is a
/// hack.
pub(crate) fn parse_enum(
    e: &ItemEnum,
    target_os: &[String],
) -> Result<RustItem, ParseErrorWithSpan> {
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
            decorators: get_decorators(&e.attrs),
            is_redacted: is_redacted(&e.attrs),
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
        .inspect(|v| debug!("\t\tChecking variant {}", v.ident))
        // Filter out variants we've been told to skip
        .filter(|v| !is_skipped(&v.attrs, target_os))
        .inspect(|v| debug!("\t\taccepted variant {}", v.ident))
        .map(|v| parse_enum_variant(v, &serde_rename_all, target_os))
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
        is_redacted: is_redacted(&e.attrs),
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
            }
            .with_span(e.span()));
        }
        if maybe_content_key.is_some() {
            return Err(ParseError::SerdeContentNotAllowed {
                enum_ident: original_enum_ident,
            }
            .with_span(e.span()));
        }

        Ok(RustItem::Enum(RustEnum::Unit(shared)))
    } else {
        // At least one enum variant is either a tuple or an anonymous struct

        let tag_key = maybe_tag_key.unwrap_or_default();
        let content_key = maybe_content_key.unwrap_or_default();

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
    target_os: &[String],
) -> Result<RustEnumVariant, ParseErrorWithSpan> {
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
                return Err(
                    ParseError::MultipleUnnamedAssociatedTypes.with_span(associated_type.span())
                );
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
                .filter(|f| !is_skipped(&f.attrs, target_os))
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
                .collect::<Result<Vec<_>, ParseErrorWithSpan>>()?,
            shared,
        }),
    }
}

/// Parses a type alias into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
pub(crate) fn parse_type_alias(t: &ItemType) -> Result<RustItem, ParseErrorWithSpan> {
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
        decorators: get_decorators(&t.attrs),
        is_redacted: is_redacted(&t.attrs),
    }))
}

/// Parses a const variant.
pub(crate) fn parse_const(c: &ItemConst) -> Result<RustItem, ParseErrorWithSpan> {
    let expr = parse_const_expr(&c.expr)?;

    // serialized_as needs to be supported in case the user wants to use a different type
    // for the constant variable in a different language
    let ty = if let Some(ty) = get_serialized_as_type(&c.attrs) {
        ty.parse()?
    } else {
        RustType::try_from(c.ty.as_ref())?
    };

    match &ty {
        RustType::Special(SpecialRustType::HashMap(_, _))
        | RustType::Special(SpecialRustType::Vec(_))
        | RustType::Special(SpecialRustType::Option(_)) => {
            return Err(ParseError::RustConstTypeInvalid.with_span(c.span()));
        }
        RustType::Special(_) => (),
        RustType::Simple { .. } => (),
        _ => {
            return Err(ParseError::RustConstTypeInvalid.with_span(c.span()));
        }
    };

    Ok(RustItem::Const(RustConst {
        id: get_ident(Some(&c.ident), &c.attrs, &None),
        r#type: ty,
        expr,
    }))
}

fn parse_const_expr(e: &Expr) -> Result<RustConstExpr, ParseErrorWithSpan> {
    struct ExprLitVisitor(pub Option<Result<RustConstExpr, ParseErrorWithSpan>>);
    impl Visit<'_> for ExprLitVisitor {
        fn visit_expr_lit(&mut self, el: &ExprLit) {
            if self.0.is_some() {
                // should we throw an error instead of silently ignoring a second literal?
                // or would this create false positives?
                return;
            }
            let check_literal_type = || {
                Ok(match &el.lit {
                    Lit::Int(lit_int) => {
                        let int: i128 = lit_int
                            .base10_parse()
                            .map_err(|_| ParseError::RustConstTypeInvalid)?;
                        RustConstExpr::Int(int)
                    }
                    _ => return Err(ParseError::RustConstTypeInvalid),
                })
            };

            self.0
                .replace(check_literal_type().map_err(|err| err.with_span(el.span())));
        }
    }
    let mut expr_visitor = ExprLitVisitor(None);
    syn::visit::visit_expr(&mut expr_visitor, e);
    expr_visitor
        .0
        .ok_or_else(|| ParseError::RustConstTypeInvalid.with_span(e.span()))?
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
#[inline(always)]
pub(crate) fn get_meta_items(attr: &syn::Attribute, ident: &str) -> impl Iterator<Item = Meta> {
    if attr.path().is_ident(ident) {
        Either::Left(
            attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .into_iter()
                .flat_map(|punctuated| punctuated.into_iter()),
        )
    } else {
        Either::Right(std::iter::empty())
    }
}

fn get_ident(
    ident: Option<&proc_macro2::Ident>,
    attrs: &[syn::Attribute],
    rename_all: &Option<String>,
) -> Id {
    let original = ident.map_or("???".to_string(), |id| id.to_string().replace("r#", ""));

    let mut renamed = rename_all_to_case(original.clone(), rename_all);

    let mut renamed_via_serde_rename = false;
    if let Some(s) = serde_rename(attrs) {
        renamed = s;
        renamed_via_serde_rename = true;
    }

    Id {
        original,
        renamed,
        serde_rename: renamed_via_serde_rename,
    }
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
fn is_skipped(attrs: &[syn::Attribute], target_os: &[String]) -> bool {
    let typeshare_skip = attrs.iter().any(|attr| {
        get_meta_items(attr, SERDE)
            .chain(get_meta_items(attr, TYPESHARE))
            .any(|arg| matches!(arg, Meta::Path(path) if path.is_ident("skip")))
    });

    typeshare_skip || !accept_target_os(attrs, target_os)
}

// `#[typeshare(redacted)]`
fn is_redacted(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        get_meta_items(attr, TYPESHARE)
            .any(|arg| matches!(arg, Meta::Path(path) if path.is_ident("redacted")))
    })
}

fn serde_attr(attrs: &[syn::Attribute], ident: &str) -> bool {
    attrs.iter().any(|attr| {
        get_meta_items(attr, SERDE)
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
        .filter_map(|meta| {
            if let Meta::List(list) = meta {
                Some(list)
            } else {
                None
            }
        })
        .filter_map(|list: MetaList| match list.path.get_ident() {
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
