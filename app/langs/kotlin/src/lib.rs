use std::{
    borrow::Cow,
    collections::HashMap,
    io::{self, Write as _},
};

use anyhow::Context;
use indent_write::io::IndentWriter;
use itertools::Itertools as _;
use joinery::JoinableIterator as _;
use lazy_format::lazy_format;
use serde::{Deserialize, Serialize};

use typeshare_model::{
    decorator::{DecoratorSet, Value},
    prelude::*,
};

enum Visibility {
    Public,
    Private,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config<'config> {
    /// Name of the output kotlin package
    #[serde(default)]
    package: Option<&'config str>,

    /// The prefix to prepend to user-defined types
    #[serde(default)]
    prefix: &'config str,

    /// Conversions from Rust type names to Kotlin type names
    #[serde(default, alias = "type_mappings")]
    type_mappings: HashMap<&'config str, &'config str>,

    /// If set, omit the headers from generated code. Usually this is set for
    /// snapshot tests.
    #[serde(default)]
    no_version_header: bool,
}

#[derive(Debug)]
pub struct Kotlin<'config> {
    package: &'config str,
    prefix: &'config str,
    type_mappings: HashMap<&'config str, &'config str>,
    no_version_header: bool,
}

impl Kotlin<'_> {
    fn write_comments(&self, w: &mut impl io::Write, comments: &[String]) -> anyhow::Result<()> {
        comments
            .iter()
            .try_for_each(|comment| writeln!(w, "/// {}", comment.trim_end()))?;

        Ok(())
    }

    fn write_element(
        &self,
        w: &mut impl io::Write,
        f: &RustField,
        generic_context: &[TypeName],
        requires_serial_name: bool,
        visibility: Visibility,
    ) -> anyhow::Result<()> {
        self.write_comments(w, &f.comments)
            .context("error writing comments")?;

        if requires_serial_name {
            writeln!(w, "@SerialName(\"{}\")", &f.id.renamed)?;
        }

        let ty = match f.decorators.type_override_for_lang("kotlin") {
            Some(ty) => ty.to_owned(),
            None => self
                .format_type(&f.ty, generic_context)
                .context("failed to format type for field")?,
        };

        let vis = match visibility {
            Visibility::Public => "",
            Visibility::Private => "private ",
        };

        let option_suffix = match (f.has_default, f.ty.is_optional()) {
            (true, false) => "? = null",
            (_, true) => " = null",
            _ => "",
        };

        let name = f.id.renamed.as_str().replace("-", "_");

        write!(w, "{vis}val {name}: {ty}{option_suffix}",)?;

        Ok(())
    }

    fn write_enum_variants(&self, w: &mut impl io::Write, e: &RustEnum) -> anyhow::Result<()> {
        match e {
            RustEnum::Unit { unit_variants, .. } => {
                for variant in unit_variants {
                    self.write_comments(w, &variant.comments)?;

                    writeln!(w, "@SerialName(\"{}\")", &variant.id.renamed)?;
                    writeln!(w, "{}(\"{}\"),", &variant.id.original, variant.id.renamed)?;
                }
            }
            RustEnum::Algebraic {
                content_key,
                variants,
                ..
            } => {
                for v in variants {
                    let printed_value = format!(r##""{}""##, &v.shared().id.renamed);
                    self.write_comments(w, &v.shared().comments)?;
                    writeln!(w, "@Serializable")?;
                    writeln!(w, "@SerialName({})", printed_value)?;

                    let variant_name = {
                        let mut variant_name = to_pascal_case(v.shared().id.original.as_str());

                        if variant_name.starts_with(|c: char| c.is_ascii_digit()) {
                            // If the name starts with a digit just add an underscore
                            // to the front and make it valid
                            variant_name = format!("_{}", variant_name);
                        }

                        variant_name
                    };

                    match v {
                        RustEnumVariant::Unit(_) => {
                            write!(w, "object {}", variant_name)?;
                        }
                        RustEnumVariant::Tuple { ty, .. } => {
                            write!(
                                w,
                                "data class {}{}(",
                                variant_name,
                                (!e.shared().generic_types.is_empty())
                                    .then(|| format!("<{}>", e.shared().generic_types.join(", ")))
                                    .unwrap_or_default()
                            )?;
                            let variant_type = self
                                .format_type(ty, e.shared().generic_types.as_slice())
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                            write!(w, "val {}: {}", content_key, variant_type)?;
                            write!(w, ")")?;
                        }
                        RustEnumVariant::AnonymousStruct { shared, fields } => {
                            write!(
                                w,
                                "data class {}{}(",
                                variant_name,
                                (!e.shared().generic_types.is_empty())
                                    .then(|| format!("<{}>", e.shared().generic_types.join(", ")))
                                    .unwrap_or_default()
                            )?;

                            // Builds the list of generic types (e.g [T, U, V]), by digging
                            // through the fields recursively and comparing against the
                            // enclosing enum's list of generic parameters.
                            let generics = fields
                                .iter()
                                .flat_map(|field| {
                                    e.shared()
                                        .generic_types
                                        .iter()
                                        .filter(|g| field.ty.contains_type(g))
                                })
                                .unique()
                                .collect_vec();

                            // Sadly the parenthesis are required because of macro limitations
                            let generics = lazy_format!(match (generics.is_empty()) {
                                false => ("<{}>", generics.iter().join_with(", ")),
                                true => (""),
                            });

                            write!(
                                w,
                                "val {}: {}{}{}Inner{}",
                                content_key,
                                self.prefix,
                                e.shared().id.original,
                                shared.id.original,
                                generics,
                            )?;
                            write!(w, ")")?;
                        }
                        variant => anyhow::bail!("unsupported variant kind {variant:?}"),
                    }

                    writeln!(
                        w,
                        ": {}{}{}()",
                        self.prefix,
                        e.shared().id.original,
                        (!e.shared().generic_types.is_empty())
                            .then(|| format!("<{}>", e.shared().generic_types.join(", ")))
                            .unwrap_or_default()
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl<'config> Language<'config> for Kotlin<'config> {
    type Config = Config<'config>;

    const NAME: &'static str = "kotlin";

    fn new_from_config(config: Self::Config) -> anyhow::Result<Self> {
        Ok(Self {
            package: config.package.context("kotlin must have a package name")?,
            prefix: config.prefix,
            type_mappings: config.type_mappings,
            no_version_header: config.no_version_header,
        })
    }

    fn output_filename_for_crate(&self, crate_name: &CrateName) -> String {
        format!("{crate_name}.kt")
    }

    fn mapped_type(&self, type_name: &TypeName) -> Option<Cow<'_, str>> {
        self.type_mappings
            .get(type_name.as_str())
            .map(|&name| Cow::Borrowed(name))
    }

    fn begin_file(
        &self,
        w: &mut impl io::Write,
        mode: FilesMode<&CrateName>,
    ) -> anyhow::Result<()> {
        let package = self.package;

        if !self.no_version_header {
            writeln!(w, "/**")?;
            writeln!(w, " * Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
            writeln!(w, " */")?;
            writeln!(w)?;
        }

        match mode {
            FilesMode::Single => writeln!(w, "package {package}",)?,
            FilesMode::Multi(crate_name) => writeln!(w, "package {package}.{crate_name}")?,
            _ => anyhow::bail!("unsupported output mode {mode:?}"),
        }

        writeln!(w)?;
        writeln!(w, "import kotlinx.serialization.Serializable")?;
        writeln!(w, "import kotlinx.serialization.SerialName")?;
        writeln!(w)?;

        Ok(())
    }

    fn format_simple_type(
        &self,
        base: &TypeName,
        generic_context: &[TypeName],
    ) -> anyhow::Result<String> {
        Ok(if generic_context.contains(base) {
            base.to_string()
        } else if let Some(name) = self.mapped_type(base) {
            name.into_owned()
        } else {
            let prefix = self.prefix;

            format!("{prefix}{base}")
        })
    }

    fn format_special_type(
        &self,
        special_ty: &SpecialRustType,
        generic_context: &[TypeName],
    ) -> anyhow::Result<String> {
        Ok(match special_ty {
            SpecialRustType::Vec(rtype) => {
                format!("List<{}>", self.format_type(rtype, generic_context)?)
            }
            SpecialRustType::Array(rtype, _) => {
                format!("List<{}>", self.format_type(rtype, generic_context)?)
            }
            SpecialRustType::Slice(rtype) => {
                format!("List<{}>", self.format_type(rtype, generic_context)?)
            }
            SpecialRustType::Option(rtype) => {
                format!("{}?", self.format_type(rtype, generic_context)?)
            }
            SpecialRustType::HashMap(rtype1, rtype2) => {
                format!(
                    "HashMap<{}, {}>",
                    self.format_type(rtype1, generic_context)?,
                    self.format_type(rtype2, generic_context)?
                )
            }
            SpecialRustType::Unit => "Unit".into(),
            // Char in Kotlin is 16 bits long, so we need to use String
            SpecialRustType::String | SpecialRustType::Char => "String".into(),
            // https://kotlinlang.org/docs/basic-types.html#integer-types
            SpecialRustType::I8 => "Byte".into(),
            SpecialRustType::I16 => "Short".into(),
            SpecialRustType::ISize | SpecialRustType::I32 => "Int".into(),
            SpecialRustType::I54 | SpecialRustType::I64 => "Long".into(),
            // https://kotlinlang.org/docs/basic-types.html#unsigned-integers
            SpecialRustType::U8 => "UByte".into(),
            SpecialRustType::U16 => "UShort".into(),
            SpecialRustType::USize | SpecialRustType::U32 => "UInt".into(),
            SpecialRustType::U53 | SpecialRustType::U64 => "ULong".into(),
            SpecialRustType::Bool => "Boolean".into(),
            SpecialRustType::F32 => "Float".into(),
            SpecialRustType::F64 => "Double".into(),
            ty => anyhow::bail!("unsupported special type {ty:?}"),
        })
    }

    fn write_imports<'a, Crates, Types>(
        &self,
        writer: &mut impl io::Write,
        _crate_name: &CrateName,
        imports: Crates,
    ) -> anyhow::Result<()>
    where
        Crates: IntoIterator<Item = (&'a CrateName, Types)>,
        Types: IntoIterator<Item = &'a TypeName>,
    {
        let package = self.package;

        for (crate_name, types) in imports {
            for ty in types {
                writeln!(writer, "import {package}.{crate_name}.{ty}")?;
            }
        }

        writeln!(writer)?;

        Ok(())
    }

    fn write_type_alias(
        &self,
        w: &mut impl io::Write,
        alias: &RustTypeAlias,
    ) -> anyhow::Result<()> {
        self.write_comments(w, &alias.comments)?;
        let type_name = format!("{}{}", &self.prefix, alias.id.original);

        if is_inline(&alias.decorators) {
            writeln!(w, "@Serializable")?;
            writeln!(w, "@JvmInline")?;
            writeln!(w, "value class {}{}(", self.prefix, alias.id.renamed)?;

            {
                self.write_element(
                    &mut IndentWriter::new("\t", &mut *w),
                    &RustField {
                        id: Id {
                            original: TypeName::new_string(String::from("value")),
                            renamed: TypeName::new_string(String::from("value")),
                        },
                        ty: alias.ty.clone(),
                        comments: vec![],
                        has_default: false,
                        decorators: DecoratorSet::new(),
                    },
                    &[],
                    false,
                    match alias.decorators.is_redacted() {
                        true => Visibility::Private,
                        false => Visibility::Public,
                    },
                )?;
            }

            writeln!(w)?;

            if alias.decorators.is_redacted() {
                writeln!(w, ") {{")?;
                writeln!(w, "\tfun unwrap() = value")?;
                writeln!(w)?;
                writeln!(w, "\toverride fun toString(): String = \"***\"")?;
                writeln!(w, "}}")?;
            } else {
                writeln!(w, ")")?;
            }

            writeln!(w)?;
        } else {
            writeln!(
                w,
                "typealias {}{} = {}\n",
                type_name,
                (!alias.generic_types.is_empty())
                    .then(|| format!("<{}>", alias.generic_types.join(", ")))
                    .unwrap_or_default(),
                self.format_type(&alias.ty, alias.generic_types.as_slice())
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            )?;
        }

        Ok(())
    }

    fn write_struct(&self, w: &mut impl io::Write, rs: &RustStruct) -> anyhow::Result<()> {
        self.write_comments(w, &rs.comments)?;
        writeln!(w, "@Serializable")?;

        if rs.fields.is_empty() {
            // If the struct has no fields, we can define it as an static object.
            writeln!(w, "object {}{}\n", self.prefix, rs.id.renamed)?;
        } else {
            writeln!(
                w,
                "data class {}{}{} (",
                self.prefix,
                rs.id.renamed,
                (!rs.generic_types.is_empty())
                    .then(|| format!("<{}>", rs.generic_types.join(", ")))
                    .unwrap_or_default()
            )?;

            {
                let mut w = IndentWriter::new("\t", &mut *w);

                // Use @SerialName when writing the struct
                //
                // As of right now this was only written to handle fields
                // that get renamed to an ident with - in it
                let requires_serial_name = rs
                    .fields
                    .iter()
                    .any(|f| f.id.renamed.as_str().contains(|c: char| c == '-'));

                if let Some((last, elements)) = rs.fields.split_last() {
                    for f in elements.iter() {
                        self.write_element(
                            &mut w,
                            f,
                            rs.generic_types.as_slice(),
                            requires_serial_name,
                            Visibility::Public,
                        )?;
                        writeln!(w, ",")?;
                    }

                    self.write_element(
                        &mut w,
                        last,
                        rs.generic_types.as_slice(),
                        requires_serial_name,
                        Visibility::Public,
                    )?;
                    writeln!(w)?;
                }
            }

            if rs.decorators.is_redacted() {
                writeln!(w, ") {{")?;
                writeln!(
                    w,
                    "\toverride fun toString(): String = \"{}\"",
                    rs.id.renamed
                )?;
                writeln!(w, "}}")?;
            } else {
                writeln!(w, ")")?;
            }

            writeln!(w)?;
        }
        Ok(())
    }

    fn write_enum(&self, w: &mut impl io::Write, e: &RustEnum) -> anyhow::Result<()> {
        // Generate named types for any anonymous struct variants of this enum
        self.write_struct_types_for_enum_variants(w, e, &|variant_name| {
            format!("{}{}Inner", &e.shared().id.renamed, variant_name)
        })?;

        self.write_comments(w, &e.shared().comments)?;
        writeln!(w, "@Serializable")?;

        let generic_parameters = (!e.shared().generic_types.is_empty())
            .then(|| format!("<{}>", e.shared().generic_types.join(", ")))
            .unwrap_or_default();

        match e {
            RustEnum::Unit { .. } => {
                write!(
                    w,
                    "enum class {}{}{}(val string: String) ",
                    self.prefix,
                    &e.shared().id.renamed,
                    generic_parameters
                )?;
            }
            RustEnum::Algebraic { .. } => {
                write!(
                    w,
                    "sealed class {}{}{} ",
                    self.prefix,
                    &e.shared().id.renamed,
                    generic_parameters
                )?;
            }
        }

        writeln!(w, "{{")?;

        {
            let mut w = IndentWriter::new("\t", &mut *w);
            self.write_enum_variants(&mut w, e)?;
        }

        writeln!(w, "}}\n")?;

        Ok(())
    }

    fn write_const(&self, _w: &mut impl io::Write, _c: &RustConst) -> anyhow::Result<()> {
        anyhow::bail!("consts aren't supported in kotlin yet")
    }

    fn exclude_from_import_analysis(&self, name: &TypeName) -> bool {
        self.type_mappings.contains_key(name.as_str())
    }
}

fn is_inline(decorators: &DecoratorSet) -> bool {
    decorators.any("kotlin", |value| match value {
        Value::String(s) => s == "JvmInline",
        _ => false,
    })
}

fn to_pascal_case(value: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize = true;
    let to_lowercase = {
        // Check if string is all uppercase, such as "URL" or "TOTP". In that case, we don't want
        // to preserve the cases.
        value.chars().all(|c| c.is_uppercase())
    };

    for ch in value.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            pascal.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            pascal.push(if to_lowercase {
                ch.to_ascii_lowercase()
            } else {
                ch
            });
        }
    }
    pascal
}
