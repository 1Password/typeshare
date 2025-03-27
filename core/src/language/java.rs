// TODO: NotNull annotations?

use convert_case::{Case, Casing as _};
use itertools::Itertools as _;

use super::{used_imports, CrateTypes, Language, ScopedCrateTypes};
use crate::language::SupportedLanguage;
use crate::parser::ParsedData;
use crate::rust_types::{RustConst, RustEnum, RustField, RustItem, RustStruct, RustTypeAlias};
use crate::rust_types::{RustEnumShared, RustTypeFormatError, SpecialRustType};
use crate::topsort::topsort;
use std::io::BufWriter;
use std::{collections::HashMap, io::Write};

/// All information needed for Java type-code
#[derive(Default)]
pub struct Java {
    /// Allow multiple classes per file
    pub allow_multiple_classes_per_file: bool,
    /// Name of the Java package
    pub package: String,
    /// Name of the Java module
    pub module_name: String,
    /// The prefix to append to user-defined types
    pub prefix: String,
    /// Conversions from Rust type names to Java type names.
    pub type_mappings: HashMap<String, String>,
    /// Whether or not to exclude the version header that normally appears at the top of generated code.
    /// If you aren't generating a snapshot test, this setting can just be left as a default (false)
    pub no_version_header: bool,
}

impl Language for Java {
    /// Given `data`, generate type-code for this language and write it out to `writable`.
    /// Returns whether or not writing was successful.
    fn generate_types(
        &mut self,
        writable: &mut dyn Write,
        all_types: &CrateTypes,
        data: ParsedData,
    ) -> std::io::Result<()> {
        self.begin_file(writable, &data)?;

        if data.multi_file {
            self.write_imports(writable, used_imports(&data, all_types))?;
        }

        let ParsedData {
            structs,
            enums,
            aliases,
            consts,
            crate_name,
            ..
        } = data;

        let namespace_class_name = crate_name.as_str().to_case(Case::Pascal);

        writeln!(writable, "public class {namespace_class_name} {{")?;
        writeln!(writable)?;

        let mut items = Vec::from_iter(
            aliases
                .into_iter()
                .map(RustItem::Alias)
                .chain(structs.into_iter().map(RustItem::Struct))
                .chain(enums.into_iter().map(RustItem::Enum))
                .chain(consts.into_iter().map(RustItem::Const)),
        );

        topsort(&mut items);

        for thing in &items {
            let mut thing_writer = BufWriter::new(Vec::new());

            match thing {
                RustItem::Enum(e) => self.write_enum(&mut thing_writer, e)?,
                RustItem::Struct(s) => self.write_struct(&mut thing_writer, s)?,
                RustItem::Alias(a) => self.write_type_alias(&mut thing_writer, a)?,
                RustItem::Const(c) => self.write_const(&mut thing_writer, c)?,
            }

            let thing_bytes = thing_writer.into_inner()?;
            let thing = self.indent(String::from_utf8(thing_bytes).unwrap(), 1);

            writable.write(thing.as_bytes())?;
        }

        writeln!(writable, "}}")?;

        self.end_file(writable)
    }

    fn type_map(&mut self) -> &HashMap<String, String> {
        &self.type_mappings
    }

    fn format_simple_type(
        &mut self,
        base: &String,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        Ok(if let Some(mapped) = self.type_map().get(base) {
            mapped.into()
        } else if generic_types.contains(base) {
            base.into()
        } else {
            format!("{}{}", self.prefix, base)
        })
    }

    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        Ok(match special_ty {
            SpecialRustType::Vec(rtype) => {
                format!(
                    "java.util.ArrayList<{}>",
                    self.format_type(rtype, generic_types)?
                )
            }
            SpecialRustType::Array(rtype, _) => {
                format!("{}[]", self.format_type(rtype, generic_types)?)
            }
            SpecialRustType::Slice(rtype) => {
                format!("{}[]", self.format_type(rtype, generic_types)?)
            }
            SpecialRustType::Option(rtype) => self.format_type(rtype, generic_types)?,
            SpecialRustType::HashMap(rtype1, rtype2) => {
                format!(
                    "java.util.HashMap<{}, {}>",
                    self.format_type(rtype1, generic_types)?,
                    self.format_type(rtype2, generic_types)?
                )
            }
            SpecialRustType::Unit => "Void".into(),
            // https://docs.oracle.com/javase/specs/jls/se23/html/jls-4.html#jls-IntegralType
            // Char in Java is 16 bits long, so we need to use String
            SpecialRustType::String | SpecialRustType::Char => "String".into(),
            SpecialRustType::I8 => "byte".into(),
            SpecialRustType::I16 => "short".into(),
            SpecialRustType::ISize | SpecialRustType::I32 => "int".into(),
            SpecialRustType::I54 | SpecialRustType::I64 => "long".into(),
            // byte in Java is signed, so we need to use short to represent all possible values
            SpecialRustType::U8 => "short".into(),
            // short in Java is signed, so we need to use int to represent all possible values
            SpecialRustType::U16 => "int".into(),
            // ing in Java is signed, so we need to use long to represent all possible values
            SpecialRustType::USize | SpecialRustType::U32 => "long".into(),
            // long in Java is signed, so we need to use BigInteger to represent all possible values
            SpecialRustType::U53 | SpecialRustType::U64 => "java.math.BigInteger".into(),
            // https://docs.oracle.com/javase/specs/jls/se23/html/jls-4.html#jls-PrimitiveType
            SpecialRustType::Bool => "boolean".into(),
            // https://docs.oracle.com/javase/specs/jls/se23/html/jls-4.html#jls-FloatingPointType
            SpecialRustType::F32 => "float".into(),
            SpecialRustType::F64 => "double".into(),
            // TODO: https://github.com/1Password/typeshare/issues/237
            SpecialRustType::DateTime => {
                return Err(RustTypeFormatError::UnsupportedSpecialType(
                    special_ty.to_string(),
                ))
            }
        })
    }

    fn begin_file(&mut self, w: &mut dyn Write, parsed_data: &ParsedData) -> std::io::Result<()> {
        if !self.package.is_empty() {
            if !self.no_version_header {
                writeln!(w, "/**")?;
                writeln!(w, " * Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
                writeln!(w, " */")?;
                writeln!(w)?;
            }
            if parsed_data.multi_file {
                writeln!(w, "package {}.{};", self.package, parsed_data.crate_name)?;
            } else {
                writeln!(w, "package {};", self.package)?;
            }
            writeln!(w)?;
        }

        Ok(())
    }

    fn write_type_alias(&mut self, _w: &mut dyn Write, _ty: &RustTypeAlias) -> std::io::Result<()> {
        todo!()
    }

    fn write_const(&mut self, _w: &mut dyn Write, _c: &RustConst) -> std::io::Result<()> {
        todo!()
    }

    fn write_struct(&mut self, w: &mut dyn Write, rs: &RustStruct) -> std::io::Result<()> {
        self.write_comments(w, 0, &rs.comments)?;

        write!(
            w,
            "public record {}{}{}(",
            self.prefix,
            rs.id.renamed,
            (!rs.generic_types.is_empty())
                .then(|| format!("<{}>", rs.generic_types.join(", ")))
                .unwrap_or_default()
        )?;

        if let Some((last, elements)) = rs.fields.split_last() {
            writeln!(w)?;
            for f in elements.iter() {
                self.write_element(w, f, rs.generic_types.as_slice())?;
                writeln!(w, ",")?;
            }
            self.write_element(w, last, rs.generic_types.as_slice())?;
            writeln!(w)?;
        }

        writeln!(w, r") {{}}")?;

        writeln!(w)?;

        Ok(())
    }

    fn write_enum(&mut self, w: &mut dyn Write, e: &RustEnum) -> std::io::Result<()> {
        // Generate named types for any anonymous struct variants of this enum
        self.write_types_for_anonymous_structs(w, e, &|variant_name| {
            format!("{}{}Inner", &e.shared().id.renamed, variant_name)
        })?;

        self.write_comments(w, 0, &e.shared().comments)?;

        match e {
            RustEnum::Unit(e) => self.write_unit_enum(w, e),
            RustEnum::Algebraic { .. } => todo!(),
        }
    }

    fn write_imports(
        &mut self,
        w: &mut dyn Write,
        imports: ScopedCrateTypes<'_>,
    ) -> std::io::Result<()> {
        for (path, ty) in imports {
            for t in ty {
                writeln!(w, "import {}.{path}.{t};", self.package)?;
            }
        }
        writeln!(w)
    }

    fn ignored_reference_types(&self) -> Vec<&str> {
        self.type_mappings.keys().map(|s| s.as_str()).collect()
    }
}

impl Java {
    #[inline]
    fn is_java_letter(&self, c: char) -> bool {
        // https://docs.oracle.com/javase/specs/jls/se23/html/jls-3.html#jls-JavaLetter
        c.is_ascii_alphabetic() || c == '_' || c == '$'
    }

    #[inline]
    fn is_java_letter_or_number(&self, c: char) -> bool {
        // https://docs.oracle.com/javase/specs/jls/se23/html/jls-3.html#jls-JavaLetterOrDigit
        self.is_java_letter(c) || c.is_ascii_digit()
    }

    #[inline]
    fn is_java_reserved_keyword(&self, name: &str) -> bool {
        // https://docs.oracle.com/javase/specs/jls/se23/html/jls-3.html#jls-ReservedKeyword
        matches!(
            name,
            "abstract"
                | "continue"
                | "for"
                | "new"
                | "switch"
                | "assert"
                | "default"
                | "if"
                | "package"
                | "synchronized"
                | "boolean"
                | "do"
                | "goto"
                | "private"
                | "this"
                | "break"
                | "double"
                | "implements"
                | "protected"
                | "throw"
                | "byte"
                | "else"
                | "import"
                | "public"
                | "throws"
                | "case"
                | "enum"
                | "instanceof"
                | "return"
                | "transient"
                | "catch"
                | "extends"
                | "int"
                | "short"
                | "try"
                | "char"
                | "final"
                | "interface"
                | "static"
                | "void"
                | "class"
                | "finally"
                | "long"
                | "strictfp"
                | "volatile"
                | "const"
                | "float"
                | "native"
                | "super"
                | "while"
                | "_"
        )
    }

    #[inline]
    fn is_java_boolean_literal(&self, name: &str) -> bool {
        // https://docs.oracle.com/javase/specs/jls/se23/html/jls-3.html#jls-BooleanLiteral
        matches!(name, "true" | "false")
    }

    #[inline]
    fn is_java_null_literal(&self, name: &str) -> bool {
        // https://docs.oracle.com/javase/specs/jls/se23/html/jls-3.html#jls-NullLiteral
        matches!(name, "null")
    }

    fn santitize_itentifier(&self, name: &str) -> String {
        // https://docs.oracle.com/javase/specs/jls/se23/html/jls-3.html#jls-Identifier
        let mut chars = name.chars();

        // Ensure the first character is valid "JavaLetter"
        let first_char = chars
            .next()
            .map(|c| if self.is_java_letter(c) { c } else { '_' });

        // Ensure each remaining characters is a valid "JavaLetterOrDigit"
        let rest: String = chars
            .filter_map(|c| match c {
                '-' => Some('_'),
                c if self.is_java_letter_or_number(c) => Some(c),
                _ => None,
            })
            .collect();

        // Combine and return the sanitized identifier
        let name: String = first_char.into_iter().chain(rest.chars()).collect();

        if self.is_java_reserved_keyword(&name)
            || self.is_java_boolean_literal(&name)
            || self.is_java_null_literal(&name)
        {
            format!("_{name}")
        } else {
            name
        }
    }

    fn write_element(
        &mut self,
        w: &mut dyn Write,
        f: &RustField,
        generic_types: &[String],
    ) -> std::io::Result<()> {
        self.write_comments(w, 1, &f.comments)?;
        let ty = match f.type_override(SupportedLanguage::Java) {
            Some(type_override) => type_override.to_owned(),
            None => self
                .format_type(&f.ty, generic_types)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
        };

        write!(w, "\t{} {}", ty, self.santitize_itentifier(&f.id.renamed))
    }

    fn write_unit_enum(&mut self, w: &mut dyn Write, e: &RustEnumShared) -> std::io::Result<()> {
        writeln!(w, "public enum {}{} {{", self.prefix, &e.id.renamed)?;

        if let Some((last, elements)) = e.variants.split_last() {
            for v in elements {
                self.write_comments(w, 1, &v.shared().comments)?;
                writeln!(
                    w,
                    "\t{},",
                    self.santitize_itentifier(&v.shared().id.renamed),
                )?;
            }
            writeln!(
                w,
                "\t{}",
                self.santitize_itentifier(&last.shared().id.renamed),
            )?;
        }

        writeln!(w, "}}")?;

        Ok(())
    }

    fn write_comment(
        &self,
        w: &mut dyn Write,
        indent: usize,
        comment: &str,
    ) -> std::io::Result<()> {
        writeln!(w, "{}/// {}", "\t".repeat(indent), comment)?;
        Ok(())
    }

    fn write_comments(
        &self,
        w: &mut dyn Write,
        indent: usize,
        comments: &[String],
    ) -> std::io::Result<()> {
        comments
            .iter()
            .try_for_each(|comment| self.write_comment(w, indent, comment))
    }

    fn indent(&self, str: impl AsRef<str>, count: usize) -> String {
        let indentation = "    ".repeat(count);
        str.as_ref()
            .split('\n')
            .map(|line| {
                if line.is_empty() {
                    line.to_string()
                } else {
                    format!("{indentation}{line}")
                }
            })
            .join("\n")
    }
}
