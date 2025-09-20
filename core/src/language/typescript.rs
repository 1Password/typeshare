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
use joinery::JoinableIterator;
use std::collections::{BTreeMap, BTreeSet};
use std::{
    collections::HashMap,
    io::{self, Write},
};

use super::ScopedCrateTypes;

/// All information needed to generate Typescript type-code
#[derive(Default)]
pub struct TypeScript {
    /// Mappings from Rust type names to Typescript type names
    pub type_mappings: HashMap<String, String>,
    /// Whether or not to exclude the version header that normally appears at the top of generated code.
    /// If you aren't generating a snapshot test, this setting can just be left as a default (false)
    pub no_version_header: bool,
    /// Carries the unique set of types for custom json translation
    pub types_for_custom_json_translation: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Clone)]
pub struct CustomJsonTranslationContent {
    reviver: String,
    replacer: String,
}

impl Language for TypeScript {
    fn type_map(&mut self) -> &HashMap<String, String> {
        &self.type_mappings
    }

    fn end_file(&mut self, w: &mut dyn Write) -> std::io::Result<()> {
        if !self.types_for_custom_json_translation.is_empty() {
            let custom_translation_content = self
                .types_for_custom_json_translation
                .iter()
                .filter_map(|(ts_type, ..)| self.custom_translations(ts_type))
                .collect::<Vec<CustomJsonTranslationContent>>();
            self.write_comments(w, 0, &["Custom JSON reviver and replacer functions for dynamic data transformation".to_owned(),
            "ReviverFunc is used during JSON parsing to detect and transform specific data structures".to_owned(),
            "ReplacerFunc is used during JSON serialization to modify certain values before stringifying.".to_owned(),
            "These functions allow for flexible encoding and decoding of data, ensuring that complex types are properly handled when converting between TS objects and JSON".to_owned()])?;

            return writeln!(
                w,
                r#"export const ReviverFunc = (key: string, value: unknown): unknown => {{
    {}
    return value;
}};

export const ReplacerFunc = (key: string, value: unknown): unknown => {{
    {}
    return value;
}};"#,
                custom_translation_content
                    .iter()
                    .map(|custom_json_translation| &custom_json_translation.reviver)
                    .join("\n    "),
                custom_translation_content
                    .iter()
                    .map(|custom_json_translation| &custom_json_translation.replacer)
                    .join("\n    ")
            );
        }
        Ok(())
    }
    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        if let Some(mapped) = self.type_mappings.get(&special_ty.to_string()) {
            if self.custom_translations(mapped).is_some() {
                self.types_for_custom_json_translation
                    .insert(mapped.to_string(), BTreeSet::new());
            }
            return Ok(mapped.to_owned());
        }
        match special_ty {
            SpecialRustType::Vec(rtype) => {
                Ok(format!("{}[]", self.format_type(rtype, generic_types)?))
            }
            SpecialRustType::Array(rtype, len) => {
                let formatted_type = self.format_type(rtype, generic_types)?;
                Ok(format!(
                    "[{}]",
                    std::iter::repeat_n(&formatted_type, *len).join_with(", ")
                ))
            }
            SpecialRustType::Slice(rtype) => {
                Ok(format!("{}[]", self.format_type(rtype, generic_types)?))
            }
            // We add optionality above the type formatting level
            SpecialRustType::Option(rtype) => self.format_type(rtype, generic_types),
            SpecialRustType::HashMap(rtype1, rtype2) => Ok(format!(
                "Record<{}, {}>",
                match rtype1.as_ref() {
                    RustType::Simple { id } if generic_types.contains(id) => {
                        return Err(RustTypeFormatError::GenericKeyForbiddenInTS(id.clone()));
                    }
                    _ => self.format_type(rtype1, generic_types)?,
                },
                self.format_type(rtype2, generic_types)?
            )),
            SpecialRustType::Unit => Ok("undefined".into()),
            SpecialRustType::DateTime => Ok("Date".into()),
            SpecialRustType::String => Ok("string".into()),
            SpecialRustType::Char => Ok("string".into()),
            SpecialRustType::I8
            | SpecialRustType::U8
            | SpecialRustType::I16
            | SpecialRustType::U16
            | SpecialRustType::I32
            | SpecialRustType::U32
            | SpecialRustType::I54
            | SpecialRustType::U53
            | SpecialRustType::F32
            | SpecialRustType::F64 => Ok("number".into()),
            SpecialRustType::Bool => Ok("boolean".into()),
            SpecialRustType::U64
            | SpecialRustType::I64
            | SpecialRustType::ISize
            | SpecialRustType::U128
            | SpecialRustType::USize => Ok("number".into()),
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

        writeln!(
            w,
            "export type {}{} = {}{};\n",
            ty.id.renamed,
            if !ty.generic_types.is_empty() {
                format!("<{}>", ty.generic_types.join(", "))
            } else {
                Default::default()
            },
            r#type,
            if ty.r#type.is_optional() {
                " | undefined"
            } else {
                Default::default()
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
        writeln!(
            w,
            "export interface {}{} {{",
            rs.id.renamed,
            if !rs.generic_types.is_empty() {
                format!("<{}>", rs.generic_types.join(", "))
            } else {
                Default::default()
            }
        )?;

        rs.fields
            .iter()
            .try_for_each(|f| self.write_field(w, f, rs.generic_types.as_slice()))?;

        writeln!(w, "}}\n")
    }

    fn write_enum(&mut self, w: &mut dyn Write, e: &RustEnum) -> io::Result<()> {
        self.write_comments(w, 0, &e.shared().comments)?;

        let generic_parameters = if !e.shared().generic_types.is_empty() {
            format!("<{}>", e.shared().generic_types.join(", "))
        } else {
            Default::default()
        };

        match e {
            RustEnum::Unit(shared) => {
                write!(
                    w,
                    "export enum {}{} {{",
                    shared.id.renamed, generic_parameters
                )?;

                self.write_enum_variants(w, e)?;

                writeln!(w, "\n}}\n")
            }
            RustEnum::Algebraic { shared, .. } => {
                write!(
                    w,
                    "export type {}{} = ",
                    shared.id.renamed, generic_parameters
                )?;

                self.write_enum_variants(w, e)?;

                write!(w, ";")?;
                writeln!(w)?;
                writeln!(w)
            }
        }
    }

    fn write_imports(
        &mut self,
        w: &mut dyn Write,
        imports: ScopedCrateTypes<'_>,
    ) -> std::io::Result<()> {
        for (path, ty) in imports {
            write!(w, "import type {{ ")?;
            let ty_list = ty.iter().join(", ");
            write!(w, "{ty_list}")?;
            writeln!(w, " }} from \"./{path}\";")?;
        }
        writeln!(w)
    }

    fn ignored_reference_types(&self) -> Vec<&str> {
        self.type_mappings.keys().map(|s| s.as_str()).collect()
    }
}

impl TypeScript {
    fn write_enum_variants(&mut self, w: &mut dyn Write, e: &RustEnum) -> io::Result<()> {
        match e {
            // Write all the unit variants out (there can only be unit variants in
            // this case)
            RustEnum::Unit(shared) => shared.variants.iter().try_for_each(|v| match v {
                RustEnumVariant::Unit(shared) => {
                    writeln!(w)?;
                    self.write_comments(w, 1, &shared.comments)?;
                    write!(w, "\t{} = {:?},", shared.id.original, &shared.id.renamed)
                }
                _ => unreachable!(),
            }),

            // Write all the algebraic variants out (all three variant types are possible
            // here)
            RustEnum::Algebraic {
                tag_key,
                content_key,
                shared,
            } => shared.variants.iter().try_for_each(|v| {
                writeln!(w)?;
                self.write_comments(w, 1, &v.shared().comments)?;
                match v {
                    RustEnumVariant::Unit(shared) => {
                        if !tag_key.is_empty() {
                            if !content_key.is_empty() {
                                write!(
                                    w,
                                    "\t| {{ {}: {:?}, {}?: undefined }}",
                                    tag_key, shared.id.renamed, content_key
                                )
                            } else {
                                write!(w, "\t| {{ {}: {:?} }}", tag_key, shared.id.renamed)
                            }
                        } else {
                            write!(w, "\t| {:?}", shared.id.renamed)
                        }
                    }
                    RustEnumVariant::Tuple { ty, shared } => {
                        let r#type = self
                            .format_type(ty, e.shared().generic_types.as_slice())
                            .map_err(std::io::Error::other)?;
                        if !tag_key.is_empty() {
                            if !content_key.is_empty() {
                                write!(
                                    w,
                                    "\t| {{ {}: {:?}, {}{}: {} }}",
                                    tag_key,
                                    shared.id.renamed,
                                    content_key,
                                    if ty.is_optional() { "?" } else { Default::default() },
                                    r#type
                                )
                            } else {
                                panic!("Tuple variants must have a content key if they have a tag key: {:?}", shared.id.original)
                            }
                        } else {
                            write!(
                                w,
                                "\t| {{ {:?}{}: {} }}",
                                shared.id.renamed,
                                if ty.is_optional() { "?" } else { Default::default() },
                                r#type
                            )
                        }
                    }
                    RustEnumVariant::AnonymousStruct { fields, shared } => {
                        if !tag_key.is_empty() {
                            if !content_key.is_empty() {
                                writeln!(
                                    w,
                                    "\t| {{ {}: {:?}, {}: {{",
                                    tag_key, shared.id.renamed, content_key
                                )?;
                            } else {
                                panic!("Struct variants must have a content key if they have a tag key: {:?}", shared.id.original)
                            }
                        } else {
                            writeln!(w, "\t| {{ {:?}: {{", shared.id.renamed)?;
                        }
                        fields.iter().try_for_each(|f| {
                            self.write_field(w, f, e.shared().generic_types.as_slice())
                        })?;

                        write!(w, "}}")?;
                        write!(w, "}}")
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
        let ts_ty: String = match field.type_override(SupportedLanguage::TypeScript) {
            Some(type_override) => type_override.to_owned(),
            None => self
                .format_type(&field.ty, generic_types)
                .map_err(io::Error::other)?,
        };
        if self.custom_translations(&ts_ty).is_some() {
            self.types_for_custom_json_translation
                .entry(ts_ty.clone())
                .and_modify(|ids| {
                    ids.insert(field.id.renamed.clone());
                })
                .or_default()
                .insert(field.id.renamed.clone());
        }
        let optional = field.ty.is_optional() || field.has_default;
        let double_optional = field.ty.is_double_optional();
        let is_readonly = field
            .decorators
            .get(&SupportedLanguage::TypeScript)
            .filter(|v| v.iter().any(|dec| dec.name() == "readonly"))
            .is_some();
        writeln!(
            w,
            "\t{}{}{}: {}{};",
            if is_readonly {
                "readonly "
            } else {
                Default::default()
            },
            typescript_property_aware_rename(&field.id.renamed),
            if optional { "?" } else { Default::default() },
            ts_ty,
            if double_optional {
                " | null"
            } else {
                Default::default()
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

    fn custom_translations(&self, ts_type: &str) -> Option<CustomJsonTranslationContent> {
        let id = self
            .types_for_custom_json_translation
            .get(ts_type)
            .filter(|ids| !ids.is_empty())
            .map(|ids| {
                format!(
                    " && ({})",
                    ids.iter()
                        .map(|id| format!("key === \"{id}\""))
                        .join(" || ")
                )
            });

        let custom_translations = HashMap::from([(
            "Uint8Array",
            (
                CustomJsonTranslationContent{
                    reviver:     r#"if (Array.isArray(value) && value.every(v => Number.isInteger(v) && v >= 0 && v <= 255) && value.length > 0)  {
        return new Uint8Array(value);
    }"#.to_owned(),
                    replacer: r#"if (value instanceof Uint8Array) {
        return Array.from(value);
    }"#.to_owned()
                }
                    )),
                    (
                        "Date",
                        CustomJsonTranslationContent{
                            replacer: r#"if (value instanceof Date) {
        return value.toISOString();
    }"#.to_owned(),
                            reviver: format!(r#"if (typeof value === "string" && /^\d{{4}}-\d{{2}}-\d{{2}}T\d{{2}}:\d{{2}}:\d{{2}}(\.\d+)?Z$/.test(value){}) {{
        return new Date(value);
    }}"#, id.unwrap_or_default())
                        }
                    )]);

        custom_translations.get(ts_type).cloned()
    }
}

fn typescript_property_aware_rename(name: &str) -> String {
    if name.chars().any(|c| c == '-') {
        return format!("{name:?}");
    }
    name.to_string()
}
