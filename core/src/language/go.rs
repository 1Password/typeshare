use std::io::Write;

use crate::language::SupportedLanguage;
use crate::parser::ParsedData;
use crate::rename::RenameExt;
use crate::rust_types::{RustItem, RustTypeFormatError, SpecialRustType};
use crate::{
    language::Language,
    rust_types::{RustEnum, RustEnumVariant, RustField, RustStruct, RustTypeAlias},
    topsort::topsort,
};
use std::collections::{HashMap, HashSet};

use super::CrateTypes;

/// All information needed to generate Go type-code
#[derive(Default)]
pub struct Go {
    /// Name of the Go package.
    pub package: String,
    /// Conversions from Rust type names to Go type names.
    pub type_mappings: HashMap<String, String>,
    /// Abbreviations that should be fully uppercased to comply with Go's formatting rules.
    pub uppercase_acronyms: Vec<String>,
    /// Whether or not to exclude the version header that normally appears at the top of generated code.
    /// If you aren't generating a snapshot test, this setting can just be left as a default (false)
    pub no_version_header: bool,
}

impl Language for Go {
    fn generate_types(
        &mut self,
        w: &mut dyn Write,
        _imports: &CrateTypes,
        data: ParsedData,
    ) -> std::io::Result<()> {
        // Generate a list of all types that either are a struct or are aliased to a struct.
        // This is used to determine whether a type should be defined as a pointer or not.
        let mut types_mapping_to_struct =
            HashSet::from_iter(data.structs.iter().map(|s| s.id.original.clone()));

        for alias in &data.aliases {
            if types_mapping_to_struct.contains(alias.r#type.id()) {
                types_mapping_to_struct.insert(alias.id.original.clone());
            }
        }

        self.begin_file(w, &data)?;

        let ParsedData {
            structs,
            enums,
            aliases,
            ..
        } = data;

        let mut items = Vec::from_iter(
            aliases
                .into_iter()
                .map(RustItem::Alias)
                .chain(structs.into_iter().map(RustItem::Struct))
                .chain(enums.into_iter().map(RustItem::Enum)),
        );

        topsort(&mut items);

        for thing in &items {
            match thing {
                RustItem::Enum(e) => self.write_enum(w, e, &types_mapping_to_struct)?,
                RustItem::Struct(s) => self.write_struct(w, s)?,
                RustItem::Alias(a) => self.write_type_alias(w, a)?,
            }
        }

        self.end_file(w)?;

        Ok(())
    }

    fn type_map(&mut self) -> &HashMap<String, String> {
        &self.type_mappings
    }

    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        Ok(match special_ty {
            SpecialRustType::Vec(rtype) => format!("[]{}", self.format_type(rtype, generic_types)?),
            SpecialRustType::Array(rtype, len) => {
                format!("[{}]{}", len, self.format_type(rtype, generic_types)?)
            }
            SpecialRustType::Slice(rtype) => {
                format!("[]{}", self.format_type(rtype, generic_types)?)
            }
            SpecialRustType::Option(rtype) => {
                format!("*{}", self.format_type(rtype, generic_types)?)
            }
            SpecialRustType::HashMap(rtype1, rtype2) => format!(
                "map[{}]{}",
                self.format_type(rtype1, generic_types)?,
                self.format_type(rtype2, generic_types)?
            ),
            SpecialRustType::Unit => "struct{}".into(),
            SpecialRustType::String => "string".into(),
            SpecialRustType::Char => "rune".into(),
            SpecialRustType::I8
            | SpecialRustType::U8
            | SpecialRustType::U16
            | SpecialRustType::I32
            | SpecialRustType::I16
            | SpecialRustType::ISize
            | SpecialRustType::USize => "int".into(),
            SpecialRustType::U32 => "uint32".into(),
            SpecialRustType::I54 | SpecialRustType::I64 => "int64".into(),
            SpecialRustType::U53 | SpecialRustType::U64 => "uint64".into(),
            SpecialRustType::Bool => "bool".into(),
            SpecialRustType::F32 => "float32".into(),
            SpecialRustType::F64 => "float64".into(),
        })
    }

    fn begin_file(&mut self, w: &mut dyn Write, _parsed_data: &ParsedData) -> std::io::Result<()> {
        if !self.no_version_header {
            // This comment is specifically formatted to satisfy gosec's template for a generated file,
            // so the generated Go file can be ignored with `gosec -exclude-generated`.
            writeln!(
                w,
                "// Code generated by typeshare {}. DO NOT EDIT.",
                env!("CARGO_PKG_VERSION")
            )?;
        }
        writeln!(w, "package {}", self.package)?;
        writeln!(w)?;
        writeln!(w, "import \"encoding/json\"")?;
        writeln!(w)?;
        Ok(())
    }

    fn write_type_alias(&mut self, w: &mut dyn Write, ty: &RustTypeAlias) -> std::io::Result<()> {
        write_comments(w, 0, &ty.comments)?;

        writeln!(
            w,
            "type {} {}\n",
            self.acronyms_to_uppercase(&ty.id.original),
            self.format_type(&ty.r#type, &[])
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        )?;

        Ok(())
    }

    fn write_struct(&mut self, w: &mut dyn Write, rs: &RustStruct) -> std::io::Result<()> {
        write_comments(w, 0, &rs.comments)?;
        writeln!(
            w,
            "type {} struct {{",
            self.acronyms_to_uppercase(&rs.id.renamed)
        )?;

        rs.fields
            .iter()
            .try_for_each(|f| self.write_field(w, f, rs.generic_types.as_slice()))?;

        writeln!(w, "}}")
    }
}

impl Go {
    fn write_enum(
        &mut self,
        w: &mut dyn Write,
        e: &RustEnum,
        custom_structs: &HashSet<String>,
    ) -> std::io::Result<()> {
        // Make a suitable name for an anonymous struct enum variant
        let uppercase_acronyms = self.uppercase_acronyms.clone();
        let make_anonymous_struct_name = |variant_name: &str| {
            convert_acronyms_to_uppercase(
                uppercase_acronyms.clone(),
                &format!("{}{}Inner", &e.shared().id.original, variant_name),
            )
        };

        // Generate named types for any anonymous struct variants of this enum
        self.write_types_for_anonymous_structs(w, e, &make_anonymous_struct_name)?;

        write_comments(w, 0, &e.shared().comments)?;

        match e {
            RustEnum::Unit(shared) => {
                writeln!(
                    w,
                    "type {} string",
                    self.acronyms_to_uppercase(&shared.id.original)
                )?;

                write!(w, "const (")?;

                shared.variants.iter().try_for_each(|v| match v {
                    RustEnumVariant::Unit(variant_shared) => {
                        writeln!(w)?;
                        write_comments(w, 1, &variant_shared.comments)?;
                        write!(
                            w,
                            "\t{}{} {} = {:?}",
                            self.acronyms_to_uppercase(&shared.id.original),
                            self.acronyms_to_uppercase(&variant_shared.id.original),
                            self.acronyms_to_uppercase(&shared.id.original),
                            &variant_shared.id.renamed
                        )
                    }
                    _ => unreachable!(),
                })?;

                writeln!(w, "\n)")
            }
            RustEnum::Algebraic {
                tag_key,
                content_key,
                shared,
                ..
            } => {
                let struct_name = self.acronyms_to_uppercase(&shared.id.original);
                let content_field = content_key.to_string().to_camel_case();
                let tag_field = self.format_field_name(tag_key.to_string(), true);
                let struct_short_name = shared.id.original[..1].to_lowercase();
                let variant_key_type = format!(
                    "{}{}s",
                    struct_name,
                    self.acronyms_to_uppercase(tag_key).to_pascal_case()
                );

                writeln!(w, "type {} string", variant_key_type)?;
                writeln!(w, "const (")?;

                let mut decoding_cases = Vec::new();
                let mut variant_accessors = Vec::new();
                let mut variant_constructors = Vec::new();

                for v in &shared.variants {
                    let variant_name = self.acronyms_to_uppercase(&v.shared().id.original);
                    let variant_type = match v {
                        RustEnumVariant::Tuple { ty, .. } => {
                            Some(self.format_type(ty, &[]).unwrap())
                        }
                        RustEnumVariant::AnonymousStruct { .. } => {
                            Some(make_anonymous_struct_name(&variant_name))
                        }
                        RustEnumVariant::Unit(_) => None,
                    };
                    let variant_type_const = format!(
                        "{}{}Variant{}",
                        struct_name,
                        self.acronyms_to_uppercase(&tag_key.to_string().to_pascal_case()),
                        variant_name
                    );
                    decoding_cases.push(format!(
                        "\tcase {variant_type_const}:\n",
                        variant_type_const = variant_type_const
                    ));

                    if let Some(variant_type) = variant_type {
                        let (variant_pointer, variant_deref, variant_ref) =
                            match (v, custom_structs.contains(&variant_type)) {
                                (RustEnumVariant::AnonymousStruct { .. }, ..) | (.., true) => {
                                    ("*", "", "")
                                }
                                _ => ("", "*", "&"),
                            };

                        decoding_cases.push(format!(
                            "\t\tvar res {variant_type}
\t\t{short_name}.{content_field} = &res
",
                            variant_type = variant_type,
                            short_name = struct_short_name,
                            content_field = content_field,
                        ));
                        variant_accessors.push(format!(
                            r#"func ({short_name} {full_name}) {variant_name}() {variant_pointer}{variant_type} {{
	res, _ := {short_name}.{content_field}.(*{variant_type})
	return {variant_deref}res
}}
"#,
                            short_name = struct_short_name,
                            full_name = struct_name,
                            variant_name = variant_name,
                            variant_pointer = variant_pointer,
                            variant_deref = variant_deref,
                            variant_type = variant_type,
                            content_field = content_field,
                        ));
                        variant_constructors.push(format!(
                            r#"func New{variant_type_const}(content {variant_pointer}{variant_type}) {struct_name} {{
    return {struct_name}{{
        {tag_field}: {variant_type_const},
        {content_field}: {variant_ref}content,
    }}
}}
"#,
                            struct_name = struct_name,
                            tag_field = tag_field,
                            variant_type_const = variant_type_const,
                            variant_pointer = variant_pointer,
                            variant_type = variant_type,
                            variant_ref = variant_ref,
                            content_field = content_field,
                        ));
                    } else {
                        decoding_cases.push("\t\treturn nil\n".to_string());

                        variant_constructors.push(format!(
                            r#"func New{variant_type_const}() {struct_name} {{
    return {struct_name}{{
        {tag_field}: {variant_type_const},
    }}
}}
"#,
                            struct_name = struct_name,
                            tag_field = tag_field,
                            variant_type_const = variant_type_const,
                        ));
                    }

                    write_comments(w, 1, &v.shared().comments)?;
                    writeln!(
                        w,
                        "\t{} {} = {:?}",
                        variant_type_const,
                        variant_key_type,
                        &v.shared().id.original
                    )?;
                }

                writeln!(w, ")")?;

                writeln!(w, "type {} struct{{ ", struct_name)?;
                writeln!(
                    w,
                    "\t{} {} `json:{:?}`",
                    self.format_field_name(tag_key.to_string(), true),
                    variant_key_type,
                    tag_key,
                )?;
                writeln!(w, "\t{} interface{{}}", content_field)?;
                writeln!(w, "}}")?;

                writeln!(
                    w,
                    r#"
func ({short_name} *{full_name}) UnmarshalJSON(data []byte) error {{
	var enum struct {{
		Tag    {variant_key_type}   `json:"{tag_key}"`
		Content json.RawMessage `json:"{content_key}"`
	}}
	if err := json.Unmarshal(data, &enum); err != nil {{
		return err
	}}

	{short_name}.{tag_field} = enum.Tag
	switch {short_name}.{tag_field} {{
{decode_cases}
	}}
	if err := json.Unmarshal(enum.Content, &{short_name}.{content_field}); err != nil {{
		return err
	}}

	return nil
}}

func ({short_name} {full_name}) MarshalJSON() ([]byte, error) {{
    var enum struct {{
		Tag    {variant_key_type}   `json:"{tag_key}"`
		Content interface{{}} `json:"{content_key},omitempty"`
    }}
    enum.Tag = {short_name}.{tag_field}
    enum.Content = {short_name}.{content_field}
    return json.Marshal(enum)
}}

{variant_accessors}
{variant_constructors}"#,
                    short_name = struct_short_name,
                    full_name = struct_name,
                    tag_field = tag_field,
                    content_field = content_field,
                    decode_cases = decoding_cases.join(""),
                    variant_accessors = variant_accessors.join(""),
                    variant_constructors = variant_constructors.join(""),
                    content_key = content_key,
                    tag_key = tag_key,
                    variant_key_type = variant_key_type,
                )
            }
        }
    }

    fn write_field(
        &mut self,
        w: &mut dyn Write,
        field: &RustField,
        generic_types: &[String],
    ) -> std::io::Result<()> {
        fn option_symbol(optional: bool) -> &'static str {
            if optional {
                ",omitempty"
            } else {
                ""
            }
        }

        write_comments(w, 1, &field.comments)?;

        let type_name = match field.type_override(SupportedLanguage::Go) {
            Some(type_override) => type_override.to_owned(),
            None => self
                .format_type(&field.ty, generic_types)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
        };

        let go_type = self.acronyms_to_uppercase(&type_name);
        let is_optional = field.ty.is_optional() || field.has_default;
        let formatted_renamed_id = format!("{:?}", &field.id.renamed);
        let renamed_id = &formatted_renamed_id[1..formatted_renamed_id.len() - 1];
        writeln!(
            w,
            "\t{} {}{} `json:\"{}{}\"`",
            self.format_field_name(field.id.original.to_string(), true),
            (field.has_default && !field.ty.is_optional())
                .then_some("*")
                .unwrap_or_default(),
            go_type,
            renamed_id,
            option_symbol(is_optional),
        )?;

        Ok(())
    }

    // Convert any of the configured acronyms to uppercase to follow Go's formatting standard.
    // If self.uppercase_acronyms contains ID (or id), Id will get replaced by ID.
    fn acronyms_to_uppercase(&self, name: &str) -> String {
        convert_acronyms_to_uppercase(self.uppercase_acronyms.clone(), name)
    }

    fn format_field_name(&mut self, name: String, exported: bool) -> String {
        let name = if exported {
            name.to_pascal_case()
        } else {
            name
        };
        self.acronyms_to_uppercase(&name)
    }
}

fn write_comment(w: &mut dyn Write, indent: usize, comment: &str) -> std::io::Result<()> {
    writeln!(w, "{}// {}", "\t".repeat(indent), comment)?;
    Ok(())
}

fn write_comments(w: &mut dyn Write, indent: usize, comments: &[String]) -> std::io::Result<()> {
    comments
        .iter()
        .try_for_each(|comment| write_comment(w, indent, comment))
}

fn convert_acronyms_to_uppercase(uppercase_acronyms: Vec<String>, name: &str) -> String {
    let mut res = name.to_string();
    for a in &uppercase_acronyms {
        for (i, a) in name.match_indices(&a.to_string().to_pascal_case()) {
            let acronym_len = a.chars().count();

            // Only perform the replacement if the matched string is not followed by a lowercase
            // or its the end of the string.
            // This prevents replacing Identity with IDentity.
            if name
                .chars()
                .nth(i + acronym_len)
                .map(|c| !c.is_lowercase())
                .unwrap_or(true)
            {
                res.replace_range(i..i + acronym_len, &a.to_uppercase());
            }
        }
    }
    res
}
