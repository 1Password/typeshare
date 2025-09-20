use crate::{
    error::GenerationError,
    language::{Language, SupportedLanguage},
    parser::{remove_dash_from_identifier, DecoratorKind, ParsedData},
    rename::RenameExt,
    rust_types::{
        DecoratorMap, RustConst, RustEnum, RustEnumVariant, RustStruct, RustTypeAlias,
        RustTypeFormatError, SpecialRustType,
    },
};
use itertools::{Either, Itertools};
use joinery::JoinableIterator;
use lazy_format::lazy_format;
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    fs,
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

// Keywords taken from https://docs.swift.org/swift-book/ReferenceManual/LexicalStructure.html
const SWIFT_KEYWORDS: &[&str] = &[
    "associatedtype",
    "class",
    "deinit",
    "enum",
    "extension",
    "fileprivate",
    "func",
    "import",
    "init",
    "inout",
    "internal",
    "let",
    "operator",
    "private",
    "protocol",
    "public",
    "rethrows",
    "static",
    "struct",
    "subscript",
    "typealias",
    "var",
    "break",
    "case",
    "continue",
    "default",
    "defer",
    "do",
    "else",
    "fallthrough",
    "for",
    "guard",
    "if",
    "in",
    "repeat",
    "return",
    "switch",
    "where",
    "while",
    "as",
    "Any",
    "catch",
    "false",
    "is",
    "nil",
    "super",
    "self",
    "Self",
    "throw",
    "throws",
    "true",
    "try",
    "Protocol",
    "Type",
];

const CODABLE: &str = "Codable";

/// Information on serialization/deserialization coding keys.
/// TODO: expand on this.
#[derive(Debug)]
struct CodingKeysInfo {
    decoding_cases: Vec<String>,
    encoding_cases: Vec<String>,
    coding_keys: Vec<String>,
}

/// A container for generic constraints.
#[derive(Debug, Clone)]
pub struct GenericConstraints {
    constraints: BTreeSet<String>,
}

impl GenericConstraints {
    /// Create a container for generic constraints from a list of strings.
    /// Each string will be broken up by `&`, the syntax that Swift uses to combine constraints,
    /// and the complete list will be de-duplicated.
    pub fn from_config(constraints: Vec<String>) -> Self {
        Self {
            constraints: std::iter::once(CODABLE.into())
                .chain(constraints.into_iter().flat_map(Self::split_constraints))
                .collect(),
        }
    }
    /// Add a new constraint expression to this container.
    /// This expression will be broken up by `&`, the syntax that Swift uses to combine constraints.
    pub fn add(&mut self, constraints: String) {
        for decorator in Self::split_constraints(constraints).into_iter() {
            self.constraints.insert(decorator);
        }
    }
    /// Get an iterator over all constraints.
    pub fn get_constraints(&self) -> impl Iterator<Item = &str> {
        self.constraints.iter().map(|s| s.as_str())
    }

    fn split_constraints(constraints: String) -> Vec<String> {
        constraints
            .split('&')
            .map(|s| s.trim().to_owned())
            .collect()
    }
}

impl Default for GenericConstraints {
    fn default() -> Self {
        Self::from_config(vec![])
    }
}

/// All information needed to generate Swift type-code
#[derive(Default)]
pub struct Swift {
    /// The prefix to append to user-defined types
    pub prefix: String,
    /// Type mappings from Rust type names to Swift type names
    pub type_mappings: HashMap<String, String>,
    /// Default decorators that will be applied to all typeshared types
    pub default_decorators: Vec<String>,
    /// Default type constraints that will be applied to all generic parameters of typeshared types
    pub default_generic_constraints: GenericConstraints,
    /// Will be set to true if one of your typeshared Rust type contains the unit type `()`.
    /// This will add a definition of a `CodableVoid` type to the generated Swift code and
    /// use `CodableVoid` to replace `()`.
    pub should_emit_codable_void: AtomicBool,
    /// Whether or not to exclude the version header that normally appears at the top of generated code.
    /// If you aren't generating a snapshot test, this setting can just be left as a default (false)
    pub no_version_header: bool,
    /// Are we generating multiple modules?
    pub multi_file: bool,
    /// The constraints to apply to `CodableVoid`.
    pub codablevoid_constraints: Vec<String>,
}

impl Language for Swift {
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
            SpecialRustType::Vec(rtype) => format!("[{}]", self.format_type(rtype, generic_types)?),
            SpecialRustType::Array(rtype, _) => {
                format!("[{}]", self.format_type(rtype, generic_types)?)
            }
            SpecialRustType::Slice(rtype) => {
                format!("[{}]", self.format_type(rtype, generic_types)?)
            }
            SpecialRustType::Option(rtype) => {
                format!("{}?", self.format_type(rtype, generic_types)?)
            }
            SpecialRustType::HashMap(rtype1, rtype2) => format!(
                "[{}: {}]",
                self.format_type(rtype1, generic_types)?,
                self.format_type(rtype2, generic_types)?
            ),
            SpecialRustType::Unit => {
                self.should_emit_codable_void.store(true, Ordering::SeqCst);
                "CodableVoid".into()
            }
            SpecialRustType::String => "String".into(),
            SpecialRustType::Char => "Unicode.Scalar".into(),
            SpecialRustType::I8 => "Int8".into(),
            SpecialRustType::U8 => "UInt8".into(),
            SpecialRustType::I16 => "Int16".into(),
            SpecialRustType::U16 => "UInt16".into(),
            SpecialRustType::USize => "UInt".into(),
            SpecialRustType::ISize => "Int".into(),
            SpecialRustType::I32 => "Int32".into(),
            SpecialRustType::U32 => "UInt32".into(),
            SpecialRustType::I54 | SpecialRustType::I64 => "Int64".into(),
            SpecialRustType::U53 | SpecialRustType::U64 => "UInt64".into(),
            SpecialRustType::Bool => "Bool".into(),
            SpecialRustType::F32 => "Float".into(),
            SpecialRustType::F64 => "Double".into(),
            // TODO: https://github.com/1Password/typeshare/issues/237
            SpecialRustType::DateTime => {
                return Err(RustTypeFormatError::UnsupportedSpecialType(
                    special_ty.to_string(),
                ))
            }
            SpecialRustType::U128 => "UInt128".into(),
        })
    }

    fn begin_file(&mut self, w: &mut dyn Write, _parsed_data: &ParsedData) -> io::Result<()> {
        if !self.no_version_header {
            writeln!(w, "/*")?;
            writeln!(w, " Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
            writeln!(w, " */")?;
            writeln!(w)?;
        }
        writeln!(w, "import Foundation")?;
        Ok(())
    }

    fn end_file(&mut self, w: &mut dyn Write) -> io::Result<()> {
        if self.should_emit_codable_void.load(Ordering::SeqCst) && !self.multi_file {
            self.write_codable(w, &self.get_codable_contents())?;
        }

        Ok(())
    }

    fn write_type_alias(&mut self, w: &mut dyn Write, ty: &RustTypeAlias) -> io::Result<()> {
        writeln!(w)?;
        self.write_comments(w, 0, &ty.comments)?;

        let swift_prefix = &self.prefix;
        let type_name = swift_keyword_aware_rename(format!("{}{}", swift_prefix, ty.id.renamed));

        writeln!(
            w,
            "public typealias {}{} = {}",
            type_name,
            if !ty.generic_types.is_empty() {
                format!("<{}>", ty.generic_types.join(", "))
            } else {
                Default::default()
            },
            self.format_type(&ty.r#type, ty.generic_types.as_slice())
                .map_err(std::io::Error::other)?
        )?;

        Ok(())
    }

    fn write_const(&mut self, _w: &mut dyn Write, _c: &RustConst) -> std::io::Result<()> {
        todo!()
    }

    fn write_struct(&mut self, w: &mut dyn Write, rs: &RustStruct) -> io::Result<()> {
        let mut coding_keys = vec![];
        let mut should_write_coding_keys = false;

        writeln!(w)?;
        self.write_comments(w, 0, &rs.comments)?;

        let type_name = swift_keyword_aware_rename(format!("{}{}", self.prefix, rs.id.renamed));

        // If there are no decorators found for this struct, still write `Codable` and default decorators for structs
        // Check if this struct's decorators contains swift in the hashmap
        let decs = if let Some(swift_decs) = rs.decorators.get(&DecoratorKind::Swift) {
            // For reach item in the received decorators in the typeshared struct add it to the original vector
            // this avoids duplicated of `Codable` without needing to `.sort()` then `.dedup()`
            // Note: the list received from `rs.decorators` is already deduped
            Either::Left(
                self.get_default_decorators().chain(
                    swift_decs
                        .iter()
                        .filter(|d| d.as_str() != CODABLE)
                        .map(|s| s.as_str()),
                ),
            )
        } else {
            Either::Right(self.get_default_decorators())
        }
        .join(", ");

        let generic_names_and_constraints =
            self.generic_constraints(&rs.decorators, &rs.generic_types);

        writeln!(
            w,
            "public struct {type_name}{}: {} {{",
            if !rs.generic_types.is_empty() {
                format!("<{generic_names_and_constraints}>",)
            } else {
                Default::default()
            },
            decs
        )?;

        for f in &rs.fields {
            self.write_comments(w, 1, &f.comments)?;

            // Create coding keys for serialization / deserialization
            //
            // As of right now this was only written to handle fields
            // that get renamed to an ident with - in it
            if f.id.renamed.chars().any(|c| c == '-') {
                coding_keys.push(format!(
                    r##"{} = "{}""##,
                    remove_dash_from_identifier(swift_keyword_aware_rename(&f.id.renamed).as_ref()),
                    &f.id.renamed
                ));

                // We only need to write out coding keys if we encounter a
                // situation like this
                should_write_coding_keys = true;
            } else {
                coding_keys.push(remove_dash_from_identifier(
                    swift_keyword_aware_rename(&f.id.renamed).as_ref(),
                ));
            }

            let case_type: String = match f.type_override(SupportedLanguage::Swift) {
                Some(type_override) => type_override.to_owned(),
                None => self
                    .format_type(&f.ty, rs.generic_types.as_slice())
                    .map_err(io::Error::other)?,
            };

            writeln!(
                w,
                "\tpublic let {}: {}{}",
                remove_dash_from_identifier(swift_keyword_aware_rename(&f.id.renamed).as_ref()),
                case_type,
                if f.has_default && !f.ty.is_optional() {
                    "?"
                } else {
                    Default::default()
                }
            )?;
        }

        if should_write_coding_keys {
            writeln!(
                w,
                r#"
	enum CodingKeys: String, CodingKey, Codable {{
		case {}
	}}"#,
                coding_keys.join(",\n\t\t\t"),
            )?;
        }

        if !rs.fields.is_empty() {
            writeln!(w)?;
        }

        let mut init_params: Vec<String> = Vec::new();
        for f in &rs.fields {
            let swift_ty = match f.type_override(SupportedLanguage::Swift) {
                Some(type_override) => type_override.to_owned(),
                None => self
                    .format_type(&f.ty, rs.generic_types.as_slice())
                    .map_err(io::Error::other)?,
            };

            init_params.push(format!(
                "{}: {}{}",
                remove_dash_from_identifier(&f.id.renamed),
                swift_ty,
                if f.has_default && !f.ty.is_optional() {
                    "?"
                } else {
                    Default::default()
                }
            ));
        }

        write!(w, "\tpublic init({}) {{", init_params.join(", "))?;
        for f in &rs.fields {
            write!(
                w,
                "\n\t\tself.{} = {}",
                remove_dash_from_identifier(&f.id.renamed),
                remove_dash_from_identifier(swift_keyword_aware_rename(&f.id.renamed).as_ref())
            )?;
        }
        if !rs.fields.is_empty() {
            write!(w, "\n\t")?;
        }
        writeln!(w, "}}")?;
        writeln!(w, "}}")?;

        Ok(())
    }

    fn write_enum(&mut self, w: &mut dyn Write, e: &RustEnum) -> io::Result<()> {
        /// Determines the decorators needed for an enum given an array of decorators
        /// that should always be present
        fn determine_decorators<'a>(
            always_present: &'a [&str],
            e: &'a RustEnum,
        ) -> impl Iterator<Item = &'a str> {
            always_present.iter().copied().chain(
                if let Some(swift_decs) = e.shared().decorators.get(&DecoratorKind::Swift) {
                    // Add any decorators from the typeshared enum
                    // Note: `swift_decs` is already deduped
                    Either::Left(
                        swift_decs
                            .iter()
                            .map(|s| s.as_str())
                            // Avoids needing to sort / dedup
                            .filter(|d| !always_present.contains(d)),
                    )
                } else {
                    Either::Right(std::iter::empty())
                },
            )
        }

        let shared = e.shared();
        let enum_name = swift_keyword_aware_rename(format!("{}{}", self.prefix, shared.id.renamed));
        let always_present = match e {
            RustEnum::Unit(_) => ["String"]
                .into_iter()
                .chain(self.get_default_decorators())
                .collect::<Vec<_>>(),
            RustEnum::Algebraic { .. } => self.get_default_decorators().collect::<Vec<_>>(),
        };
        let decs = determine_decorators(&always_present, e).join(", ");

        // Make a suitable name for an anonymous struct enum variant
        let make_anonymous_struct_name =
            |variant_name: &str| format!("{}{}Inner", shared.id.renamed, variant_name);

        writeln!(w)?;

        // Generate named types for any anonymous struct variants of this enum
        self.write_types_for_anonymous_structs(w, e, &make_anonymous_struct_name)?;

        self.write_comments(w, 0, &shared.comments)?;
        let indirect = if shared.is_recursive { "indirect " } else { "" };

        let generic_names_and_constraints =
            self.generic_constraints(&e.shared().decorators, &e.shared().generic_types);

        writeln!(
            w,
            "public {indirect}enum {enum_name}{}: {} {{",
            if !e.shared().generic_types.is_empty() {
                format!("<{generic_names_and_constraints}>",)
            } else {
                Default::default()
            },
            decs
        )?;

        let coding_keys_info = self.write_enum_variants(w, e, make_anonymous_struct_name)?;

        if !coding_keys_info.coding_keys.is_empty() {
            writeln!(
                w,
                r#"
	enum CodingKeys: String, CodingKey, Codable {{
		case {}
	}}"#,
                coding_keys_info.coding_keys.join(",\n\t\t\t"),
            )?;
        }

        if let RustEnum::Algebraic {
            tag_key,
            content_key,
            ..
        } = e
        {
            writeln!(
                w,
                r#"
	private enum ContainerCodingKeys: String, CodingKey {{
		case {tag_key}, {content_key}
	}}

	public init(from decoder: Decoder) throws {{
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .{tag_key}) {{
			switch type {{{decoding_switch}
			}}
		}}
		throw DecodingError.typeMismatch({type_name}.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for {type_name}"))
	}}

	public func encode(to encoder: Encoder) throws {{
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {{{encoding_switch}
		}}
	}}"#,
                tag_key = tag_key,
                content_key = content_key,
                type_name = enum_name,
                decoding_switch = coding_keys_info.decoding_cases.join(""),
                encoding_switch = coding_keys_info.encoding_cases.join(""),
            )?;
        }

        writeln!(w, "}}")
    }

    // TODO: This will be added in the future.
    // fn write_imports(
    //     &mut self,
    //     w: &mut dyn Write,
    //     imports: super::ScopedCrateTypes<'_>,
    // ) -> std::io::Result<()> {
    //     for module in imports.keys() {
    //         writeln!(w, "import {}", module.0.to_pascal_case())?;
    //     }
    //     writeln!(w)
    // }

    fn write_imports(
        &mut self,
        _writer: &mut dyn Write,
        _imports: super::ScopedCrateTypes<'_>,
    ) -> std::io::Result<()> {
        // This will be added to Foundation for now.
        Ok(())
    }

    fn post_generation(&self, output_folder: &str) -> Result<(), GenerationError> {
        //
        if self.should_emit_codable_void.load(Ordering::SeqCst) && self.multi_file {
            self.write_codable_file(output_folder)
                .map_err(|e| GenerationError::PostGeneration(e.to_string()))?;
        }
        Ok(())
    }
}

impl Swift {
    fn write_enum_variants(
        &mut self,
        w: &mut dyn Write,
        e: &RustEnum,
        make_anonymous_struct_name: impl Fn(&str) -> String,
    ) -> io::Result<CodingKeysInfo> {
        let mut decoding_cases = Vec::new();
        let mut encoding_cases = Vec::new();
        let mut coding_keys = Vec::new();

        match e {
            RustEnum::Unit(shared) => {
                for v in &shared.variants {
                    let variant_name = v.shared().id.original.to_camel_case();

                    self.write_comments(w, 1, &v.shared().comments)?;
                    if v.shared().id.renamed == variant_name {
                        // We don't need to handle any renaming
                        writeln!(w, "\tcase {}", &swift_keyword_aware_rename(&variant_name))?;
                    } else {
                        // We do need to handle renaming
                        writeln!(
                            w,
                            "\tcase {} = {:?}",
                            swift_keyword_aware_rename(&variant_name),
                            &v.shared().id.renamed
                        )?;
                    }
                }
            }
            RustEnum::Algebraic {
                tag_key,
                content_key,
                shared,
            } => {
                let generics = &shared.generic_types;
                for v in &shared.variants {
                    self.write_comments(w, 1, &v.shared().comments)?;

                    let variant_name = {
                        let mut variant_name = v.shared().id.original.to_camel_case();

                        if variant_name
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                        {
                            // If the name starts with a digit just add an underscore
                            // to the front and make it valid
                            variant_name = format!("_{variant_name}");
                        }

                        variant_name
                    };

                    coding_keys.push(if variant_name == v.shared().id.renamed {
                        swift_keyword_aware_rename(&variant_name).into_owned()
                    } else {
                        format!(
                            r##"{} = "{}""##,
                            swift_keyword_aware_rename(&variant_name),
                            &v.shared().id.renamed
                        )
                    });

                    write!(w, "\tcase {}", swift_keyword_aware_rename(&variant_name))?;

                    match v {
                        RustEnumVariant::Unit(_) => {
                            decoding_cases.push(format!(
                                "
			case .{case_name}:
				self = .{case_name}
				return",
                                case_name = &variant_name,
                            ));

                            encoding_cases.push(format!(
                                "
		case .{case_name}:
			try container.encode(CodingKeys.{case_name}, forKey: .{tag_key})",
                                tag_key = tag_key,
                                case_name = swift_keyword_aware_rename(&variant_name),
                            ));
                        }
                        RustEnumVariant::Tuple { ty, .. } => {
                            let content_optional = ty.is_optional();
                            let case_type = self
                                .format_type(ty, e.shared().generic_types.as_slice())
                                .map_err(io::Error::other)?;
                            write!(w, "({})", swift_keyword_aware_rename(&case_type))?;

                            if content_optional {
                                decoding_cases.push(format!(
                                    "
            case .{case_name}:
				if let content = try? container.decode({case_type}.self, forKey: .{content_key}) {{
					self = .{case_name}(content)
					return
				}}
				else if let isNil = try? container.decodeNil(forKey: .{content_key}), isNil {{
					self = .{case_name}(nil)
					return
				}}",
                                    content_key = content_key,
                                    case_type = swift_keyword_aware_rename(&case_type),
                                    case_name = &variant_name
                                ))
                            } else {
                                decoding_cases.push(format!(
                                    "
			case .{case_name}:
				if let content = try? container.decode({case_type}.self, forKey: .{content_key}) {{
					self = .{case_name}(content)
					return
				}}",
                                    content_key = content_key,
                                    case_type = swift_keyword_aware_rename(&case_type),
                                    case_name = &variant_name,
                                ));
                            }

                            encoding_cases.push(format!(
                                "
		case .{case_name}(let content):
			try container.encode(CodingKeys.{case_name}, forKey: .{tag_key})
			try container.encode(content, forKey: .{content_key})",
                                tag_key = tag_key,
                                content_key = content_key,
                                case_name = &variant_name,
                            ));
                        }
                        RustEnumVariant::AnonymousStruct { shared, fields } => {
                            let anonymous_struct_name = format!(
                                "{}{}",
                                self.prefix,
                                make_anonymous_struct_name(&shared.id.original)
                            );

                            // Builds the list of generic types (e.g [T, U, V]), by digging
                            // through the fields recursively and comparing against the
                            // enclosing enum's list of generic parameters.
                            let generic_types = fields
                                .iter()
                                .flat_map(|field| {
                                    generics.iter().filter(|g| field.ty.contains_type(g))
                                })
                                .unique()
                                .collect_vec();

                            // Sadly the parenthesis are required because of macro limitations
                            let generic_types = lazy_format!(match (generic_types.is_empty()) {
                                false => ("<{}>", generic_types.iter().join_with(", ")),
                                true => (""),
                            });

                            write!(w, "({}{})", &anonymous_struct_name, generic_types)?;

                            decoding_cases.push(format!(
                                "
			case .{case_name}:
				if let content = try? container.decode({case_type}{generic_list}.self, forKey: .{content_key}) {{
					self = .{case_name}(content)
					return
				}}",
                                content_key = content_key,
                                case_type = &anonymous_struct_name,
                                case_name = &variant_name,
                                generic_list = &generic_types,
                            ));

                            encoding_cases.push(format!(
                                "
		case .{case_name}(let content):
			try container.encode(CodingKeys.{case_name}, forKey: .{tag_key})
			try container.encode(content, forKey: .{content_key})",
                                tag_key = tag_key,
                                content_key = content_key,
                                case_name = &variant_name,
                            ));
                        }
                    }

                    writeln!(w)?;
                }
            }
        }

        Ok(CodingKeysInfo {
            decoding_cases,
            encoding_cases,
            coding_keys,
        })
    }

    fn write_comment(&mut self, w: &mut dyn Write, indent: usize, comment: &str) -> io::Result<()> {
        writeln!(w, "{}/// {}", "\t".repeat(indent), comment.trim_end())?;
        Ok(())
    }

    fn write_comments(
        &mut self,
        w: &mut dyn Write,
        indent: usize,
        comments: &[String],
    ) -> io::Result<()> {
        comments
            .iter()
            .try_for_each(|c| self.write_comment(w, indent, c))
    }
}

impl Swift {
    fn get_default_decorators(&self) -> impl Iterator<Item = &str> {
        [CODABLE]
            .into_iter()
            .chain(self.default_decorators.iter().map(|s| s.as_str()))
    }

    /// When using multiple file generation we write this into a separate module vs at the
    /// end of the generated file.
    fn write_codable_file(&self, output_folder: &str) -> std::io::Result<()> {
        let output_string = self.get_codable_contents();
        let output_path = Path::new(output_folder).join("Codable.swift");

        if let Ok(buf) = fs::read(&output_path) {
            if buf == output_string.as_bytes() {
                return Ok(());
            }
        }

        let mut w = fs::File::create(output_path)?;
        self.write_codable(&mut w, &output_string)
    }

    fn get_codable_contents(&self) -> String {
        let mut decs = self
            .get_default_decorators()
            .chain(self.codablevoid_constraints.iter().map(|s| s.as_str()))
            .collect::<Vec<_>>();

        // If there are no decorators found for this struct, still write `Codable` and default decorators for structs
        if !decs.contains(&CODABLE) {
            decs.push(CODABLE);
        }

        format!("\n/// () isn't codable, so we use this instead to represent Rust's unit type\npublic struct CodableVoid: {} {{}}", decs.join(", "))
    }

    /// Write the `CodableVoid` type.
    fn write_codable(&self, w: &mut dyn Write, output_string: &str) -> io::Result<()> {
        writeln!(w, "{output_string}")
    }

    /// Build the generic constraints output. This checks for the `swiftGenericConstraints` typeshare attribute and combines
    /// it with the `default_generic_constraints` configuration. If no `swiftGenericConstraints` is defined then we just use
    /// `default_generic_constraints`.
    fn generic_constraints<'a>(
        &'a self,
        decorator_map: &'a DecoratorMap,
        generic_types: &'a [String],
    ) -> String {
        let swift_generic_constraints_annotated = decorator_map
            .get(&DecoratorKind::SwiftGenericConstraints)
            .map(|generic_constraints| {
                generic_constraints
                    .iter()
                    .filter_map(|generic_constraint| {
                        let mut gen_name_val_iter = generic_constraint.split(':');
                        let generic_name = gen_name_val_iter.next()?;
                        let mut generic_name_constraints = gen_name_val_iter
                            .next()?
                            .split('&')
                            .map(|s| s.trim())
                            .collect::<BTreeSet<_>>();
                        // Merge default generic constraints with annotated constraints.
                        generic_name_constraints
                            .extend(self.default_generic_constraints.get_constraints());
                        Some((generic_name, generic_name_constraints))
                    })
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();

        generic_types
            .iter()
            .map(
                |type_name| match swift_generic_constraints_annotated.get(type_name.as_str()) {
                    // Use constraints from swiftGenericConstraints decorator.
                    Some(constraints) => (type_name, Either::Left(constraints.iter().copied())),
                    // Use the default generic constraints if it is not part of a swiftGenericConstraints decorator.
                    None => (
                        type_name,
                        Either::Right(self.default_generic_constraints.get_constraints()),
                    ),
                },
            )
            .map(|(type_name, mut constraints)| format!("{type_name}: {}", constraints.join(" & ")))
            .join(", ")
    }
}

fn swift_keyword_aware_rename<'a, T>(name: T) -> Cow<'a, str>
where
    T: Into<Cow<'a, str>>,
{
    let name = name.into();
    if SWIFT_KEYWORDS.contains(&name.as_ref()) {
        Cow::Owned(format!("`{name}`"))
    } else {
        name
    }
}
