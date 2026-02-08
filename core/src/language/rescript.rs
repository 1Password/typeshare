use crate::rust_types::Id;
use crate::RenameExt;
use crate::{
    language::{Language, SupportedLanguage},
    parser::ParsedData,
    rust_types::{
        RustConst, RustConstExpr, RustEnum, RustEnumVariant, RustField, RustStruct, RustType,
        RustTypeAlias, RustTypeFormatError, SpecialRustType,
    },
};
use itertools::Itertools;
use std::{
    borrow::Cow,
    collections::HashMap,
    io::{self, Write},
};

use super::ScopedCrateTypes;

// Source: https://rescript-lang.org/docs/manual/reserved-keywords/
const RESCRIPT_KEYWORDS: &[&str] = &[
    "and",
    "as",
    "assert",
    "constraint",
    "else",
    "exception",
    "external",
    "false",
    "for",
    "if",
    "in",
    "include",
    "lazy",
    "let",
    "module",
    "mutable",
    "of",
    "open",
    "rec",
    "switch",
    "true",
    "try",
    "type",
    "when",
    "while",
    "with",
];

/// All information needed to generate ReScript type-code
#[derive(Default)]
pub struct ReScript {
    /// Mappings from Rust type names to ReScript type names
    pub type_mappings: HashMap<String, String>,
    /// Default decorators that will be applied to all typeshared types
    pub default_decorators: Vec<String>,
    /// Whether or not to exclude the version header that normally appears at the top of generated code.
    /// If you aren't generating a snapshot test, this setting can just be left as a default (false)
    pub no_version_header: bool,
}

impl Language for ReScript {
    fn type_map(&mut self) -> &HashMap<String, String> {
        &self.type_mappings
    }

    fn end_file(&mut self, _w: &mut dyn Write) -> std::io::Result<()> {
        Ok(())
    }

    /// Format a simple type with no generic parameters.
    /// Note that we still need to take a list of generic types in case the implementors
    /// need to differentiate between a user-defined type and a generic type (for example: Swift)
    #[allow(clippy::ptr_arg)]
    fn format_simple_type(
        &mut self,
        base: &String,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        if let Some(mapped) = self.type_map().get(base) {
            Ok(mapped.into())
        } else {
            let base_camel = base.to_camel_case();
            // If this is a generic type parameter (i.e., it's in the generic_types list),
            // prefix it with an apostrophe as required by ReScript
            if generic_types.contains(base) {
                Ok(format!("'{}", base_camel))
            } else {
                Ok(base_camel)
            }
        }
    }

    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        if let Some(mapped) = self.type_mappings.get(&special_ty.to_string()) {
            // if self.custom_translations(mapped).is_some() {
            //     self.types_for_custom_json_translation
            //         .insert(mapped.to_string(), BTreeSet::new());
            // }
            return Ok(mapped.to_owned());
        }
        match special_ty {
            SpecialRustType::Vec(rtype) => Ok(format!(
                "array<{}>",
                self.format_type(rtype, generic_types)?
            )),
            SpecialRustType::Array(rtype, len) => {
                // ReScript doesn't have a fixed-length array type, so we just use a regular array type and add a comment about the length
                let formatted_type = self.format_type(rtype, generic_types)?;
                Ok(format!("array<{}> /* length: {} */", formatted_type, len))
            }
            SpecialRustType::Slice(rtype) => Ok(format!(
                "array<{}>",
                self.format_type(rtype, generic_types)?
            )),
            // We add optionality above the type formatting level
            SpecialRustType::Option(rtype) => self.format_type(rtype, generic_types),
            SpecialRustType::HashMap(rtype1, rtype2) => {
                let _hashtype = match rtype1.as_ref() {
                    RustType::Simple { id } if generic_types.contains(id) => {
                        return Err(RustTypeFormatError::GenericKeyForbiddenInTS(id.clone()));
                    }
                    _ => self.format_type(rtype1, generic_types)?,
                };
                Ok(format!(
                    "Dict.t<{}>",
                    self.format_type(rtype2, generic_types)?
                ))
            }
            SpecialRustType::Unit => Ok("unit".into()),
            SpecialRustType::DateTime => Ok("Date.t".into()),
            SpecialRustType::String => Ok("string".into()),
            SpecialRustType::Char => Ok("char".into()),
            SpecialRustType::I8
            | SpecialRustType::U8
            | SpecialRustType::I16
            | SpecialRustType::U16
            | SpecialRustType::I32
            | SpecialRustType::U32
            | SpecialRustType::I54
            | SpecialRustType::U53 => Ok("int".into()),
            SpecialRustType::F32 | SpecialRustType::F64 => Ok("float".into()),
            SpecialRustType::Bool => Ok("bool".into()),
            SpecialRustType::U64
            | SpecialRustType::I64
            | SpecialRustType::ISize
            | SpecialRustType::USize => {
                panic!("64 bit types not allowed in Typeshare")
            }
        }
    }

    fn begin_file(&mut self, w: &mut dyn Write, _parsed_data: &ParsedData) -> io::Result<()> {
        if !self.no_version_header {
            writeln!(w, "/*")?;
            writeln!(w, " Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
            writeln!(w, "*/")?;
            writeln!(w)?;
        }
        Ok(())
    }

    fn write_type_alias(&mut self, w: &mut dyn Write, ty: &RustTypeAlias) -> io::Result<()> {
        self.write_comments(w, 0, &ty.comments)?;

        let r#type = self
            .format_type(&ty.r#type, ty.generic_types.as_slice())
            .map_err(io::Error::other)?;

        // Apply default decorators if any
        if !self.default_decorators.is_empty() {
            writeln!(w, "{}", self.default_decorators.join(" "))?;
        }

        writeln!(
            w,
            "type {}{} = {}\n",
            rescript_keyword_aware_rename(ty.id.renamed.to_camel_case()),
            if !ty.generic_types.is_empty() {
                format!(
                    "<{}>",
                    ty.generic_types
                        .iter()
                        .map(|s| format!("'{}", s.to_camel_case()))
                        .join(", ")
                )
            } else {
                Default::default()
            },
            if ty.r#type.is_optional() {
                format!("option<{}>", r#type)
            } else {
                r#type
            },
        )?;

        Ok(())
    }

    fn write_const(&mut self, w: &mut dyn Write, c: &RustConst) -> io::Result<()> {
        match c.expr {
            RustConstExpr::Int(val) => {
                let const_type = self
                    .format_type(&c.r#type, &[])
                    .map_err(std::io::Error::other)?;
                writeln!(
                    w,
                    "export const {}: {} = {};",
                    c.id.renamed.to_snake_case().to_uppercase(),
                    const_type,
                    val
                )
            }
        }
    }

    fn write_struct(&mut self, w: &mut dyn Write, rs: &RustStruct) -> io::Result<()> {
        self.write_comments(w, 0, &rs.comments)?;

        // Apply default decorators if any
        if !self.default_decorators.is_empty() {
            writeln!(w, "{}", self.default_decorators.join(" "))?;
        }

        let type_name = rs.id.renamed.to_camel_case();
        let generic_parameters = if !rs.generic_types.is_empty() {
            format!(
                "<{}>",
                rs.generic_types
                    .iter()
                    .map(|s| format!("'{}", s.to_camel_case()))
                    .join(", ")
            )
        } else {
            Default::default()
        };

        writeln!(
            w,
            "type {}{} = {{",
            rescript_keyword_aware_rename(type_name),
            generic_parameters
        )?;
        // writeln!(w, "type t = {{")?;

        rs.fields
            .iter()
            .try_for_each(|f| self.write_field(w, f, rs.generic_types.as_slice()))?;

        // writeln!(w, "}}\n")?;
        writeln!(w, "}}\n")
    }

    fn write_enum(&mut self, w: &mut dyn Write, e: &RustEnum) -> io::Result<()> {
        self.write_comments(w, 0, &e.shared().comments)?;

        let generic_parameters = if !e.shared().generic_types.is_empty() {
            format!(
                "<{}>",
                e.shared()
                    .generic_types
                    .iter()
                    .map(|s| format!("'{}", s.to_camel_case()))
                    .join(", ")
            )
        } else {
            Default::default()
        };

        // Apply default decorators if any
        if !self.default_decorators.is_empty() {
            writeln!(w, "{}", self.default_decorators.join(" "))?;
        }

        match e {
            RustEnum::Unit(shared) => {
                if e.shared().variants.len() == 0 {
                    // Special case: an empty enum
                    write!(
                        w,
                        "type {}{}",
                        rescript_keyword_aware_rename(shared.id.renamed.to_camel_case()),
                        generic_parameters
                    )?;
                } else {
                    write!(
                        w,
                        "type {}{} = ",
                        rescript_keyword_aware_rename(shared.id.renamed.to_camel_case()),
                        generic_parameters
                    )?;

                    self.write_enum_variants(w, e)?;
                }
                writeln!(w, "\n\n")
            }
            RustEnum::Algebraic {
                shared, tag_key, ..
            } => {
                let parent_name = shared.id.renamed.to_camel_case();
                // Write internal structs before the actual type because rescript does not allow nested structs inside enums/variants
                shared.variants.iter().try_for_each(|v| {
                    // writeln!(w)?;
                    // self.write_comments(w, 1, &v.shared().comments)?;
                    match v {
                        RustEnumVariant::Unit(_shared) => {
                            // Do nothing
                        }
                        RustEnumVariant::Tuple { ty: _, shared: _ } => {
                            // Do nothing
                        }
                        RustEnumVariant::AnonymousStruct { fields, shared } => {
                            if !self.default_decorators.is_empty() {
                                writeln!(w, "{}", self.default_decorators.join(" "))?;
                            }
                            self.write_struct(
                                w,
                                &RustStruct {
                                    id: Id {
                                        original: rescript_keyword_aware_rename(format!(
                                            "{}{}",
                                            parent_name,
                                            shared.id.original.to_pascal_case()
                                        ))
                                        .to_string(),
                                        renamed: rescript_keyword_aware_rename(format!(
                                            "{}{}",
                                            parent_name,
                                            shared.id.renamed.to_pascal_case()
                                        ))
                                        .to_string(),
                                        serde_rename: shared.id.serde_rename,
                                    },
                                    decorators: HashMap::new(),
                                    is_redacted: false,
                                    comments: shared.comments.clone(),
                                    fields: fields.to_vec(),
                                    generic_types: e.shared().generic_types.clone(),
                                },
                            )?;
                        }
                    }
                    io::Result::Ok(())
                })?;

                writeln!(w, "@tag(\"{}\")", tag_key)?;
                write!(
                    w,
                    "type {}{} = ",
                    rescript_keyword_aware_rename(shared.id.renamed.to_camel_case()),
                    generic_parameters
                )?;

                self.write_enum_variants(w, e)?;

                writeln!(w)?;
                writeln!(w)
            }
        }
    }

    fn write_imports(
        &mut self,
        _w: &mut dyn Write,
        _imports: ScopedCrateTypes<'_>,
    ) -> std::io::Result<()> {
        // ReScript does not require import statements for the generated types.
        Ok(())
    }

    fn ignored_reference_types(&self) -> Vec<&str> {
        self.type_mappings.keys().map(|s| s.as_str()).collect()
    }
}

impl ReScript {
    fn write_enum_variants(&mut self, w: &mut dyn Write, e: &RustEnum) -> io::Result<()> {
        let parent_name = e.shared().id.original.to_camel_case();
        match e {
            // Write all the unit variants out (there can only be unit variants in
            // this case)
            RustEnum::Unit(shared) => shared.variants.iter().try_for_each(|v| match v {
                RustEnumVariant::Unit(shared) => {
                    writeln!(w)?;
                    self.write_comments(w, 1, &shared.comments)?;
                    write!(
                        w,
                        "\t| @as({:?}) {}",
                        &shared.id.renamed,
                        shared.id.original.to_pascal_case()
                    )
                }
                _ => unreachable!(),
            }),

            // Write all the algebraic variants out (all three variant types are possible
            // here)
            RustEnum::Algebraic {
                tag_key: _,
                content_key,
                shared,
            } => shared.variants.iter().try_for_each(|v| {
                writeln!(w)?;
                self.write_comments(w, 1, &v.shared().comments)?;
                match v {
                    RustEnumVariant::Unit(shared) => {
                        write!(
                            w,
                            "\t| @as(\"{}\") {}",
                            shared.id.renamed,
                            shared.id.original.to_pascal_case()
                        )
                    }
                    RustEnumVariant::Tuple { ty, shared } => {
                        let r#type = self
                            .format_type(ty, e.shared().generic_types.as_slice())
                            .map_err(io::Error::other)?;
                        write!(
                            w,
                            "\t| @as(\"{}\") {}({{ {}{}: {} }})",
                            shared.id.renamed,
                            shared.id.original.to_pascal_case(),
                            content_key,
                            if ty.is_optional() {
                                "?"
                            } else {
                                Default::default()
                            },
                            r#type
                        )
                    }
                    RustEnumVariant::AnonymousStruct { fields, shared } => {
                        let generic_str = if e.shared().generic_types.as_slice().len() > 0 {
                            format!(
                                "<{}>",
                                e.shared()
                                    .generic_types
                                    .as_slice()
                                    .iter()
                                    .map(|s| format!("'{}", s.to_camel_case()))
                                    .join(", ")
                            )
                        } else {
                            Default::default()
                        };
                        write!(
                            w,
                            "\t| @as(\"{}\") {}({{ {}: {}{} }})",
                            shared.id.renamed,
                            shared.id.original.to_pascal_case(),
                            content_key,
                            format!("{}{}", parent_name, shared.id.original.to_pascal_case()),
                            generic_str
                        )
                    }
                }
            }),
        }
    }

    fn write_field(
        &mut self,
        w: &mut dyn Write,
        field: &RustField,
        generic_types: &[String],
    ) -> io::Result<()> {
        self.write_comments(w, 1, &field.comments)?;
        let ts_ty: String = match field.type_override(SupportedLanguage::ReScript) {
            Some(type_override) => type_override.to_owned(),
            None => self
                .format_type(&field.ty, generic_types)
                .map_err(io::Error::other)?,
        };
        let optional = field.ty.is_optional() || field.has_default;
        let double_optional = field.ty.is_double_optional();
        writeln!(
            w,
            "\t{}{}: {},",
            rescript_keyword_aware_rename(&field.id.renamed),
            if optional { "?" } else { Default::default() },
            if double_optional {
                format!("option<{}>", ts_ty)
            } else {
                ts_ty
            }
        )?;

        Ok(())
    }

    fn write_comments(
        &mut self,
        w: &mut dyn Write,
        indent: usize,
        comments: &[String],
    ) -> io::Result<()> {
        // Only attempt to write a comment if there are some, otherwise we're Ok()
        if !comments.is_empty() {
            let comment: String = {
                let tab_indent = "\t".repeat(indent);
                // If there's only one comment then keep it on the same line, otherwise we'll make a nice multi-line comment
                if comments.len() == 1 {
                    format!("{}/** {} */", tab_indent, comments.first().unwrap())
                } else {
                    let joined_comments = comments.join(&format!("\n{tab_indent} * "));
                    format!(
                        "{tab_indent}/**
{tab_indent} * {joined_comments}
{tab_indent} */"
                    )
                }
            };
            writeln!(w, "{comment}")?;
        }
        Ok(())
    }
}

fn rescript_keyword_aware_rename<'a, T>(name: T) -> Cow<'a, str>
where
    T: Into<Cow<'a, str>>,
{
    let name = name.into();
    if RESCRIPT_KEYWORDS.contains(&name.as_ref()) {
        Cow::Owned(format!("\\\"{name}\""))
    } else {
        // If name contains hyphen
        if name.contains('-') {
            Cow::Owned(format!("\\\"{name}\""))
        } else {
            name
        }
    }
}
