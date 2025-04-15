//! Source file parsing.
use ignore::Walk;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    path::PathBuf,
};
use syn::{
    punctuated::Punctuated, visit::Visit, Attribute, Expr, ExprGroup, ExprLit, ExprParen, Fields,
    GenericParam, Ident, ItemConst, ItemEnum, ItemStruct, ItemType, Lit, Meta, Token,
};

use typeshare_model::{
    decorator::{self, DecoratorSet},
    prelude::*,
};

use crate::{
    rename::RenameExt,
    type_parser::{parse_rust_type, parse_rust_type_from_string},
    visitors::TypeShareVisitor,
    FileParseErrors, ParseError, ParseErrorKind, ParseErrorSet,
};

const SERDE: &str = "serde";
const TYPESHARE: &str = "typeshare";

/// An enum that encapsulates units of code generation for Typeshare.
/// Analogous to `syn::Item`, even though our variants are more limited.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum RustItem {
    /// A `struct` definition
    Struct(RustStruct),
    /// An `enum` definition
    Enum(RustEnum),
    /// A `type` definition or newtype struct.
    Alias(RustTypeAlias),
    /// A `const` definition
    Const(RustConst),
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
    /// TODO: This is currently almost empty. Import computation was found to
    /// be pretty broken during the migration to Typeshare 2, so that part
    /// of multi-file output was stripped out to be restored later.
    pub import_types: HashSet<ImportedType>,
}

impl ParsedData {
    pub fn merge(&mut self, other: Self) {
        self.structs.extend(other.structs);
        self.enums.extend(other.enums);
        self.aliases.extend(other.aliases);
        self.consts.extend(other.consts);
        self.import_types.extend(other.import_types);
    }

    pub fn add(&mut self, item: RustItem) {
        match item {
            RustItem::Struct(rust_struct) => self.structs.push(rust_struct),
            RustItem::Enum(rust_enum) => self.enums.push(rust_enum),
            RustItem::Alias(rust_type_alias) => self.aliases.push(rust_type_alias),
            RustItem::Const(rust_const) => self.consts.push(rust_const),
        }
    }

    pub fn all_type_names(&self) -> impl Iterator<Item = &'_ TypeName> + use<'_> {
        let s = self.structs.iter().map(|s| &s.id.renamed);
        let e = self.enums.iter().map(|e| &e.shared().id.renamed);
        let a = self.aliases.iter().map(|a| &a.id.renamed);
        // currently we ignore consts, which aren't types. May revisit this
        // later.

        s.chain(e).chain(a)
    }

    pub fn sort_contents(&mut self) {
        self.structs
            .sort_unstable_by(|lhs, rhs| Ord::cmp(&lhs.id.original, &rhs.id.original));

        self.enums.sort_unstable_by(|lhs, rhs| {
            Ord::cmp(&lhs.shared().id.original, &rhs.shared().id.original)
        });

        self.aliases
            .sort_unstable_by(|lhs, rhs| Ord::cmp(&lhs.id.original, &rhs.id.original));

        self.consts
            .sort_unstable_by(|lhs, rhs| Ord::cmp(&lhs.id.original, &rhs.id.original));
    }
}

/// Input data for parsing each source file.
#[derive(Debug)]
pub struct ParserInput {
    /// Rust source file path.
    file_path: PathBuf,
    /// The crate name the source file belongs to, if we could detect it
    crate_name: Option<CrateName>,
}

/// Walk the source folder and collect all parser inputs.
pub fn parser_inputs(walker_builder: Walk) -> Vec<ParserInput> {
    walker_builder
        .filter_map(Result::ok)
        .filter(|dir_entry| !dir_entry.path().is_dir())
        .map(|dir_entry| {
            let path = dir_entry.path();
            let crate_name = CrateName::find_crate_name(path);
            let file_path = path.to_path_buf();

            ParserInput {
                file_path,
                crate_name,
            }
        })
        .collect()
}

// /// This function produces the `import_candidates`
// /// Collect all the typeshared types into a mapping of crate names to typeshared types. This
// /// mapping is used to lookup and generated import statements for generated files.
// pub fn all_types(file_mappings: &HashMap<CrateName, ParsedData>) -> CrateTypes {
//     file_mappings
//         .iter()
//         .map(|(crate_name, parsed_data)| (crate_name, &parsed_data.type_names))
//         .fold(
//             HashMap::new(),
//             |mut import_map: CrateTypes, (crate_name, type_names)| {
//                 match import_map.entry(crate_name.clone()) {
//                     Entry::Occupied(mut e) => {
//                         e.get_mut().extend(type_names.iter().cloned());
//                     }
//                     Entry::Vacant(e) => {
//                         e.insert(type_names.clone());
//                     }
//                 }
//                 import_map
//             },
//         )
// }

fn add_parsed_data(
    container: &mut HashMap<Option<CrateName>, ParsedData>,
    crate_name: Option<CrateName>,
    parsed_data: ParsedData,
) {
    match container.entry(crate_name) {
        Entry::Vacant(entry) => {
            entry.insert(parsed_data);
        }
        Entry::Occupied(entry) => {
            entry.into_mut().merge(parsed_data);
        }
    }
}

/// Collect all the parsed sources into a mapping of crate name to parsed data.
pub fn parse_input(
    inputs: Vec<ParserInput>,
    ignored_types: &[&str],
    mode: FilesMode<()>,
) -> Result<HashMap<Option<CrateName>, ParsedData>, Vec<FileParseErrors>> {
    inputs
        .into_par_iter()
        .map(|parser_input| {
            // Performance nit: we don't need to clone in the error case;
            // map_err is taking unconditional ownership unnecessarily
            let content = std::fs::read_to_string(&parser_input.file_path).map_err(|err| {
                FileParseErrors::new(
                    parser_input.file_path.clone(),
                    parser_input.crate_name.clone(),
                    crate::FileErrorKind::ReadError(err),
                )
            })?;

            let parsed_data = parse(
                &content,
                ignored_types,
                match mode {
                    FilesMode::Single => FilesMode::Single,
                    FilesMode::Multi(()) => match parser_input.crate_name {
                        None => {
                            return Err(FileParseErrors::new(
                                parser_input.file_path.clone(),
                                parser_input.crate_name,
                                crate::FileErrorKind::UnknownCrate,
                            ))
                        }
                        Some(ref crate_name) => FilesMode::Multi(crate_name),
                    },
                },
            )
            .map_err(|err| {
                FileParseErrors::new(
                    parser_input.file_path.clone(),
                    parser_input.crate_name.clone(),
                    crate::FileErrorKind::ParseErrors(err),
                )
            })?;

            let parsed_data = parsed_data.and_then(|parsed_data| {
                if is_parsed_data_empty(&parsed_data) {
                    None
                } else {
                    Some(parsed_data)
                }
            });

            Ok(parsed_data.map(|parsed_data| (parser_input.crate_name, parsed_data)))
        })
        .filter_map(|data| data.transpose())
        .fold(
            || Ok(HashMap::new()),
            |mut accum, result| {
                match (&mut accum, result) {
                    (Ok(accum), Ok((crate_name, parsed_data))) => {
                        add_parsed_data(accum, crate_name, parsed_data)
                    }
                    (Ok(_), Err(error)) => {
                        accum = Err(Vec::from([error]));
                    }
                    (Err(accum), Err(error)) => accum.push(error),
                    (Err(_), Ok(_)) => {}
                }

                accum
            },
        )
        .reduce(
            || Ok(HashMap::new()),
            |old, new| match (old, new) {
                (Ok(mut old), Ok(new)) => {
                    new.into_iter().for_each(|(crate_name, parsed_data)| {
                        add_parsed_data(&mut old, crate_name, parsed_data)
                    });
                    Ok(old)
                }
                (Err(errors), Ok(_)) | (Ok(_), Err(errors)) => Err(errors),
                (Err(mut err1), Err(err2)) => {
                    err1.extend(err2);
                    Err(err1)
                }
            },
        )
}

/// Check if we have not parsed any relavent typehsared types.
fn is_parsed_data_empty(parsed_data: &ParsedData) -> bool {
    parsed_data.enums.is_empty() && parsed_data.aliases.is_empty() && parsed_data.structs.is_empty()
}

/// Parse the given Rust source string into `ParsedData`.
pub fn parse(
    source_code: &str,
    ignored_types: &[&str],
    file_mode: FilesMode<&CrateName>,
) -> Result<Option<ParsedData>, ParseErrorSet> {
    // We will only produce output for files that contain the `#[typeshare]`
    // attribute, so this is a quick and easy performance win
    if !source_code.contains("#[typeshare") {
        return Ok(None);
    }

    // Parse and process the input, ensuring we parse only items marked with
    // `#[typeshare]`
    let mut import_visitor = TypeShareVisitor::new(ignored_types, file_mode);
    let file_contents = syn::parse_file(source_code)
        .map_err(|err| ParseError::new(&err.span(), ParseErrorKind::SynError(err)))?;

    import_visitor.visit_file(&file_contents);

    import_visitor.parsed_data().map(Some)
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
            GenericParam::Type(type_param) => Some(TypeName::new(&type_param.ident)),
            _ => None,
        })
        .collect();

    let decorators = get_decorators(&s.attrs);

    // Check if this struct should be parsed as a type alias.
    // TODO: we shouldn't lie and return a type alias when parsing a struct. this
    // is a temporary hack
    if let Some(ty) = get_serialized_as_type(&decorators) {
        return Ok(RustItem::Alias(RustTypeAlias {
            id: get_ident(Some(&s.ident), &s.attrs, &None),
            ty: parse_rust_type_from_string(&ty)?,
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
                    let decorators = get_decorators(&f.attrs);

                    let ty = match get_serialized_as_type(&decorators) {
                        Some(ty) => parse_rust_type_from_string(&ty)?,
                        None => parse_rust_type(&f.ty)?,
                    };

                    if serde_flatten(&f.attrs) {
                        return Err(ParseError::new(&f, ParseErrorKind::SerdeFlattenNotAllowed));
                    }

                    let has_default = serde_default(&f.attrs);

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
                decorators,
            })
        }
        // Tuple structs
        Fields::Unnamed(f) => {
            let Some(f) = f.unnamed.iter().exactly_one().ok() else {
                return Err(ParseError::new(f, ParseErrorKind::ComplexTupleStruct));
            };

            let decorators = get_decorators(&f.attrs);

            let ty = match get_serialized_as_type(&decorators) {
                Some(ty) => parse_rust_type_from_string(&ty)?,
                None => parse_rust_type(&f.ty)?,
            };

            RustItem::Alias(RustTypeAlias {
                id: get_ident(Some(&s.ident), &s.attrs, &None),
                ty: ty,
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
            decorators,
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
            GenericParam::Type(type_param) => Some(TypeName::new(&type_param.ident)),
            _ => None,
        })
        .collect();

    let serde_rename_all = serde_rename_all(&e.attrs);
    let decorators = get_decorators(&e.attrs);

    // TODO: we shouldn't lie and return a type alias when parsing an enum. this
    // is a temporary hack
    if let Some(ty) = get_serialized_as_type(&decorators) {
        return Ok(RustItem::Alias(RustTypeAlias {
            id: get_ident(Some(&e.ident), &e.attrs, &None),
            ty: parse_rust_type_from_string(&ty)?,
            comments: parse_comment_attrs(&e.attrs),
            generic_types,
        }));
    }

    let original_enum_ident = TypeName::new(&e.ident);

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
        _ => panic!("unrecgonized enum type"),
    });

    let shared = RustEnumShared {
        id: get_ident(Some(&e.ident), &e.attrs, &None),
        comments: parse_comment_attrs(&e.attrs),
        decorators,
        generic_types,
        is_recursive,
    };

    // Figure out if we're dealing with a unit enum or an algebraic enum
    if variants
        .iter()
        .all(|v| matches!(v, RustEnumVariant::Unit(_)))
    {
        // All enum variants are unit-type
        if maybe_tag_key.is_some() {
            return Err(ParseError::new(
                &e,
                ParseErrorKind::SerdeTagNotAllowed {
                    enum_ident: original_enum_ident,
                },
            ));
        }
        if maybe_content_key.is_some() {
            return Err(ParseError::new(
                &e,
                ParseErrorKind::SerdeContentNotAllowed {
                    enum_ident: original_enum_ident,
                },
            ));
        }

        Ok(RustItem::Enum(RustEnum::Unit {
            shared,
            unit_variants: variants
                .into_iter()
                .map(|variant| match variant {
                    RustEnumVariant::Unit(unit) => unit,
                    _ => unreachable!("non-unit variant; this was checked earlier"),
                })
                .collect(),
        }))
    } else {
        // At least one enum variant is either a tuple or an anonymous struct
        Ok(RustItem::Enum(RustEnum::Algebraic {
            tag_key: maybe_tag_key.ok_or_else(|| {
                ParseError::new(
                    &e,
                    ParseErrorKind::SerdeTagRequired {
                        enum_ident: original_enum_ident.clone(),
                    },
                )
            })?,
            content_key: maybe_content_key.ok_or_else(|| {
                ParseError::new(
                    &e,
                    ParseErrorKind::SerdeContentRequired {
                        enum_ident: original_enum_ident.clone(),
                    },
                )
            })?,
            shared,
            variants,
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
            let Some(field) = associated_type.unnamed.iter().exactly_one().ok() else {
                return Err(ParseError::new(
                    associated_type,
                    ParseErrorKind::MultipleUnnamedAssociatedTypes,
                ));
            };
            let decorators = get_decorators(&field.attrs);

            let ty = match get_serialized_as_type(&decorators) {
                Some(ty) => parse_rust_type_from_string(&ty)?,
                None => parse_rust_type(&field.ty)?,
            };

            Ok(RustEnumVariant::Tuple { ty, shared })
        }
        syn::Fields::Named(fields_named) => Ok(RustEnumVariant::AnonymousStruct {
            fields: fields_named
                .named
                .iter()
                .filter(|f| !is_skipped(&f.attrs))
                .map(|f| {
                    let decorators = get_decorators(&f.attrs);

                    let field_type = match get_serialized_as_type(&decorators) {
                        Some(ty) => parse_rust_type_from_string(&ty)?,
                        None => parse_rust_type(&f.ty)?,
                    };

                    let has_default = serde_default(&f.attrs);

                    Ok(RustField {
                        id: get_ident(f.ident.as_ref(), &f.attrs, &variant_serde_rename_all),
                        ty: field_type,
                        comments: parse_comment_attrs(&f.attrs),
                        has_default,
                        decorators,
                    })
                })
                .try_collect()?,
            shared,
        }),
    }
}

/// Parses a type alias into a definition that more succinctly represents what
/// typeshare needs to generate code for other languages.
pub(crate) fn parse_type_alias(t: &ItemType) -> Result<RustItem, ParseError> {
    let decorators = get_decorators(&t.attrs);

    let ty = match get_serialized_as_type(&decorators) {
        Some(ty) => parse_rust_type_from_string(&ty)?,
        None => parse_rust_type(&t.ty)?,
    };

    let generic_types = t
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(type_param) => Some(TypeName::new(&type_param.ident)),
            _ => None,
        })
        .collect();

    Ok(RustItem::Alias(RustTypeAlias {
        id: get_ident(Some(&t.ident), &t.attrs, &None),
        ty,
        comments: parse_comment_attrs(&t.attrs),
        generic_types,
    }))
}

/// Parses a const variant.
pub(crate) fn parse_const(c: &ItemConst) -> Result<RustItem, ParseError> {
    let expr = parse_const_expr(&c.expr)?;
    let decorators = get_decorators(&c.attrs);

    // serialized_as needs to be supported in case the user wants to use a different type
    // for the constant variable in a different language
    let ty = match get_serialized_as_type(&decorators) {
        Some(ty) => parse_rust_type_from_string(ty)?,
        None => parse_rust_type(&c.ty)?,
    };

    match &ty {
        RustType::Special(SpecialRustType::HashMap(_, _))
        | RustType::Special(SpecialRustType::Vec(_))
        | RustType::Special(SpecialRustType::Option(_)) => {
            return Err(ParseError::new(&c.ty, ParseErrorKind::RustConstTypeInvalid));
        }
        RustType::Special(_) => (),
        RustType::Simple { .. } => (),
        _ => return Err(ParseError::new(&c.ty, ParseErrorKind::RustConstTypeInvalid)),
    };

    Ok(RustItem::Const(RustConst {
        id: get_ident(Some(&c.ident), &c.attrs, &None),
        ty,
        expr,
    }))
}

fn parse_const_expr(e: &Expr) -> Result<RustConstExpr, ParseError> {
    let value = match e {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit), ..
        }) => lit
            .base10_parse()
            .map_err(|_| ParseError::new(&lit, ParseErrorKind::RustConstExprInvalid))?,

        Expr::Group(ExprGroup { expr, .. }) | Expr::Paren(ExprParen { expr, .. }) => {
            return parse_const_expr(expr)
        }
        _ => return Err(ParseError::new(e, ParseErrorKind::RustConstExprInvalid)),
    };

    Ok(RustConstExpr::Int(value))
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

pub(crate) fn get_serialized_as_type(decorators: &DecoratorSet) -> Option<&str> {
    // TODO: what to do if there are multiple instances of serialized_as?
    match decorators.get("serialized_as")?.first()? {
        decorator::Value::String(s) => Some(s),
        _ => None,
    }
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

fn get_ident(ident: Option<&Ident>, attrs: &[syn::Attribute], rename_all: &Option<String>) -> Id {
    let original = ident.map_or("???".to_string(), |id| id.to_string().replace("r#", ""));

    let mut renamed = rename_all_to_case(original.clone(), rename_all);

    if let Some(s) = serde_rename(attrs) {
        renamed = s;
    }

    Id {
        original: TypeName::new_string(original),
        renamed: TypeName::new_string(renamed),
    }
}

fn rename_all_to_case(original: String, case: &Option<String>) -> String {
    // TODO: we'd like to replace this with `heck`, but it's not clear that
    // we'd preserve backwards compatibility
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

/// Checks the struct or enum for decorators like `#[typeshare(typescript = "readonly")]`
/// Takes a slice of `syn::Attribute`, returns a `HashMap<language, Vec<decorator>>`, where `language` is `SupportedLanguage`
/// and `decorator` is `FieldDecorator`. Field decorators are ordered in a `BTreeSet` for consistent code generation.
fn get_decorators(attrs: &[Attribute]) -> DecoratorSet {
    attrs
        .iter()
        .flat_map(|attr| match attr.meta {
            Meta::List(ref meta) => Some(meta),
            Meta::Path(_) | Meta::NameValue(_) => None,
        })
        .filter(|meta| meta.path.is_ident(TYPESHARE))
        .filter_map(|meta| {
            meta.parse_args_with(Punctuated::<KeyMaybeValue, Token![,]>::parse_terminated)
                .ok()
        })
        .flatten()
        .fold(DecoratorSet::new(), |mut set, decorator| {
            set.entry(decorator.key).or_default().push(decorator.value);

            set
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

fn get_tag_key(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "tag", SERDE).next()
}

fn get_content_key(attrs: &[syn::Attribute]) -> Option<String> {
    get_name_value_meta_items(attrs, "content", SERDE).next()
}

/// For parsing decorators: a single `key` or `key = "value"` in an attribute,
/// where `key` is an identifier and `value` is some literal
struct KeyMaybeValue {
    key: String,
    value: decorator::Value,
}

impl syn::parse::Parse for KeyMaybeValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        let eq: Option<syn::Token![=]> = input.parse()?;
        let value: Option<syn::Lit> = eq.map(|_| input.parse()).transpose()?;

        let value = match value {
            None => decorator::Value::None,
            Some(syn::Lit::Str(lit)) => decorator::Value::String(lit.value()),
            Some(syn::Lit::Int(lit)) => decorator::Value::Int(lit.base10_parse()?),
            Some(syn::Lit::Bool(lit)) => decorator::Value::Bool(lit.value),
            Some(lit) => {
                return Err(syn::Error::new(
                    lit.span(),
                    "unsupported decorator type (need string, int, or bool)",
                ))
            }
        };

        Ok(KeyMaybeValue {
            key: key.to_string(),
            value,
        })
    }
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
