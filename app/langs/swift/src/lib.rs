use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    fs,
    io::{self, Write as _},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::Context;
use indent_write::io::IndentWriter;
use itertools::Itertools;
use joinery::{Joinable, JoinableIterator};
use lazy_format::lazy_format;
use serde::{Deserialize, Serialize};

use typeshare_model::{
    decorator::{DecoratorSet, Value},
    prelude::*,
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

/// Information on serialization/deserialization coding keys.
/// TODO: expand on this.
#[derive(Debug)]
struct CodingKeysInfo {
    decoding_cases: Vec<String>,
    encoding_cases: Vec<String>,
    coding_keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config<'a> {
    /// The prefix to apply to all swift types
    #[serde(default)]
    prefix: &'a str,

    /// The set of decorators to apply to all typeshared types
    #[serde(default)]
    default_decorators: Vec<&'a str>,

    /// The set of generic constraints that will be applied to all generic
    /// parameters of typeshared types
    #[serde(default)]
    default_generic_constraints: BTreeSet<&'a str>,

    /// The constraints to apply to `CodableVoid`.
    #[serde(default)]
    codablevoid_constraints: Vec<&'a str>,

    /// Mappings from rust type names to swift type names
    #[serde(default)]
    type_mappings: HashMap<&'a str, &'a str>,

    /// If false true, no header will be added to output swift files. Generally
    /// used for snapshot tests.
    #[serde(default)]
    no_version_header: bool,
}

#[derive(Debug)]
pub struct Swift<'a> {
    prefix: &'a str,
    default_decorators: Vec<&'a str>,
    default_generic_constraints: BTreeSet<&'a str>,
    codablevoid_constraints: Vec<&'a str>,
    type_mappings: HashMap<&'a str, &'a str>,
    no_version_header: bool,

    /// Will be set to true if one of any typeshared Rust type contains the unit type `()`.
    /// This will add a definition of a `CodableVoid` type to the generated Swift code and
    /// use `CodableVoid` to replace `()`.
    should_emit_codable_void: AtomicBool,
}

impl Swift<'_> {
    fn write_comments(&self, w: &mut impl io::Write, comments: &[String]) -> io::Result<()> {
        comments
            .iter()
            // Use .split to preserve empty trailing lines
            .flat_map(|comment| comment.split('\n'))
            .map(|comment| comment.trim_end())
            .try_for_each(|comment| writeln!(w, "/// {comment}"))
    }

    fn write_enum_variants(
        &self,
        w: &mut impl io::Write,
        e: &RustEnum,
        make_anonymous_struct_name: impl Fn(&TypeName) -> String,
    ) -> anyhow::Result<CodingKeysInfo> {
        let mut decoding_cases = Vec::new();
        let mut encoding_cases = Vec::new();
        let mut coding_keys = Vec::new();

        match e {
            RustEnum::Unit { unit_variants, .. } => {
                for v in unit_variants {
                    let variant_name = to_camel_case(v.id.original.as_str());

                    self.write_comments(&mut IndentWriter::new("\t", &mut *w), &v.comments)
                        .with_context(|| {
                            format!("failed to write comments for variant {}", v.id.original)
                        })?;

                    if v.id.renamed.as_str() == variant_name {
                        // We don't need to handle any renaming
                        writeln!(w, "\tcase {}", &swift_keyword_aware_rename(&variant_name))?;
                    } else {
                        // We do need to handle renaming
                        writeln!(
                            w,
                            "\tcase {} = {:?}",
                            swift_keyword_aware_rename(&variant_name),
                            &v.id.renamed.as_str()
                        )?;
                    }
                }
            }
            RustEnum::Algebraic {
                tag_key,
                content_key,
                shared,
                variants,
            } => {
                let generics = &shared.generic_types;
                for v in variants {
                    self.write_comments(
                        &mut IndentWriter::new("\t", &mut *w),
                        &v.shared().comments,
                    )
                    .with_context(|| {
                        format!(
                            "failed to write comments for variant {}",
                            v.shared().id.original
                        )
                    })?;

                    let variant_name = {
                        let mut variant_name = to_camel_case(v.shared().id.original.as_str());

                        if variant_name
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                        {
                            // If the name starts with a digit just add an underscore
                            // to the front and make it valid
                            variant_name = format!("_{}", variant_name);
                        }

                        variant_name
                    };

                    coding_keys.push(if variant_name == v.shared().id.renamed.as_str() {
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
                        _ => anyhow::bail!("unsupported enum variant {v:?}"),
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

    fn write_codable_void(&self, w: &mut impl io::Write) -> io::Result<()> {
        let decs = self
            .default_decorators
            .iter()
            .chain(&self.codablevoid_constraints)
            .unique()
            .join(", ");

        writeln!(
            w,
            "\n\
             /// () isn't codable, so we use this instead to represent Rust's unit type\n\
             public struct CodableVoid: {decs} {{}}"
        )
    }

    /// Get all of the decorators for a type or enum or variants. The decorators
    /// are deduplicated, and returned in this order:
    /// [extras, defaults, decorators].
    fn get_complete_decorators_for_type<'a>(
        &'a self,
        extras: &[&'a str],
        decorators: &'a DecoratorSet,
    ) -> impl Iterator<Item = &'a str> {
        let local_decorators = decorators
            .get_all("swift")
            .iter()
            .filter_map(|decorator| match decorator {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .flat_map(|decorator| decorator.split(","))
            .map(|decorator| decorator.trim());

        // Remove this line once we no longer need sorted output. The input
        // to this function is totally deterministic (source order), so
        // there should be no need to sort it.
        let local_decorators = {
            let mut dec = local_decorators.collect_vec();
            dec.sort_unstable();
            dec
        };

        let extras = extras.iter().copied();
        let default = self.default_decorators.iter().copied();

        extras.chain(default).chain(local_decorators).unique()
    }

    /// Build the generic constraints output. This checks for the `swiftGenericConstraints` typeshare attribute and combines
    /// it with the `default_generic_constraints` configuration. If no `swiftGenericConstraints` is defined then we just use
    /// `default_generic_constraints`.
    ///
    /// If there are no generic types, this returns the empty string.
    fn generic_constraints(
        &self,
        decorator_map: &DecoratorSet,
        generic_types: &[TypeName],
    ) -> String {
        if generic_types.is_empty() {
            return String::new();
        }

        let swift_generic_constraints_annotated = decorator_map
            .get_all("swiftGenericConstraints")
            .iter()
            .filter_map(|decorator| match decorator {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .flat_map(|constraints| constraints.split(","))
            .filter_map(|generic_constraint| {
                let (name, constraints) = generic_constraint.split_once(":")?;
                let constraints = constraints.split("&").map(|constraint| constraint.trim());

                let constraints = BTreeSet::from_iter(
                    constraints.chain(self.default_generic_constraints.iter().copied()),
                );

                Some((name.trim(), constraints))
            })
            .collect::<HashMap<_, _>>();

        generic_types
            .iter()
            .map(|type_name| {
                let constraint_set = swift_generic_constraints_annotated
                    .get(type_name.as_str())
                    .unwrap_or(&self.default_generic_constraints);

                let constraints = constraint_set.iter().join_with(" & ");

                let formatted = lazy_format!(match (constraint_set.is_empty()) {
                    true => "{type_name}",
                    false => "{type_name}: {constraints}",
                });

                formatted
            })
            .join(", ")
    }
}

impl<'config> Language<'config> for Swift<'config> {
    type Config = Config<'config>;

    const NAME: &'static str = "swift";

    fn new_from_config(config: Self::Config) -> anyhow::Result<Self> {
        Ok(Self {
            prefix: config.prefix,
            default_decorators: {
                let mut decorators = config.default_decorators;
                decorators.retain(|&d| d != "Codable");
                decorators.insert(0, "Codable");
                decorators
            },
            default_generic_constraints: {
                let mut constraints = config.default_generic_constraints;
                constraints.insert("Codable");
                constraints
            },
            codablevoid_constraints: config.codablevoid_constraints,
            type_mappings: config.type_mappings,
            no_version_header: config.no_version_header,
            should_emit_codable_void: AtomicBool::new(false),
        })
    }

    fn output_filename_for_crate(&self, crate_name: &CrateName) -> String {
        format!("{}.swift", to_pascal_case(crate_name.as_str()))
    }

    fn mapped_type(&self, type_name: &TypeName) -> Option<Cow<'_, str>> {
        self.type_mappings
            .get(type_name.as_str())
            .copied()
            .map(Cow::Borrowed)
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
            format!("{}{}", self.prefix, base)
        })
    }

    fn format_special_type(
        &self,
        special_ty: &SpecialRustType,
        generic_context: &[TypeName],
    ) -> anyhow::Result<String> {
        Ok(match special_ty {
            SpecialRustType::Vec(rtype) => {
                format!("[{}]", self.format_type(rtype, generic_context)?)
            }
            SpecialRustType::Array(rtype, _) => {
                format!("[{}]", self.format_type(rtype, generic_context)?)
            }
            SpecialRustType::Slice(rtype) => {
                format!("[{}]", self.format_type(rtype, generic_context)?)
            }
            SpecialRustType::Option(rtype) => {
                format!("{}?", self.format_type(rtype, generic_context)?)
            }
            SpecialRustType::HashMap(rtype1, rtype2) => format!(
                "[{}: {}]",
                self.format_type(rtype1, generic_context)?,
                self.format_type(rtype2, generic_context)?
            ),
            SpecialRustType::Unit => {
                // Relaxed is sufficient because the only time this value will
                // be read is either by *this* thread, or *after* this thread
                // is joined
                self.should_emit_codable_void.store(true, Ordering::Relaxed);
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
            _ => anyhow::bail!("unsupported type {special_ty:?}"),
        })
    }

    fn begin_file(
        &self,
        w: &mut impl io::Write,
        _mode: FilesMode<&CrateName>,
    ) -> anyhow::Result<()> {
        if !self.no_version_header {
            writeln!(w, "/*")?;
            writeln!(w, " Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
            writeln!(w, " */")?;
            writeln!(w)?;
        }
        writeln!(w, "import Foundation")?;
        Ok(())
    }

    // fn write_imports(
    //     &mut self,
    //     w: &mut dyn Write,
    //     imports: super::ScopedCrateTypes<'_>,
    // ) -> std::io::Result<()> {

    // }

    fn write_imports<'a, Crates, Types>(
        &self,
        _writer: &mut impl io::Write,
        _crate_name: &CrateName,
        _imports: Crates,
    ) -> anyhow::Result<()>
    where
        Crates: IntoIterator<Item = (&'a CrateName, Types)>,
        Types: IntoIterator<Item = &'a TypeName>,
    {
        // TODO: This will be added in the future.
        // for module in imports.keys() {
        //     writeln!(w, "import {}", module.0.to_pascal_case())?;
        // }
        // writeln!(w)

        Ok(())
    }

    fn write_type_alias(
        &self,
        w: &mut impl io::Write,
        alias: &typeshare_model::prelude::RustTypeAlias,
    ) -> anyhow::Result<()> {
        writeln!(w)?;
        self.write_comments(w, &alias.comments)?;

        let swift_prefix = &self.prefix;
        let type_name = format!("{}{}", swift_prefix, alias.id.original);
        let type_name = swift_keyword_aware_rename(&type_name);

        writeln!(
            w,
            "public typealias {}{} = {}",
            type_name,
            (!alias.generic_types.is_empty())
                .then(|| format!("<{}>", alias.generic_types.join(", ")))
                .unwrap_or_default(),
            self.format_type(&alias.ty, alias.generic_types.as_slice())
                .context("failed to format type")?,
        )
        .context("i/o error")?;

        Ok(())
    }

    fn write_struct(&self, w: &mut impl io::Write, rs: &RustStruct) -> anyhow::Result<()> {
        let mut coding_keys = vec![];
        let mut should_write_coding_keys = false;

        writeln!(w)?;
        self.write_comments(w, &rs.comments)?;

        let type_name = format!("{}{}", self.prefix, rs.id.original);
        let type_name = swift_keyword_aware_rename(&type_name);

        // If there are no decorators found for this struct, still write `Codable` and default decorators for structs
        // Check if this struct's decorators contains swift in the hashmap
        let decs = self
            .get_complete_decorators_for_type(&[], &rs.decorators)
            .join(", ");

        let generic_names_and_constraints =
            self.generic_constraints(&rs.decorators, &rs.generic_types);

        writeln!(
            w,
            "public struct {type_name}{}: {} {{",
            (!rs.generic_types.is_empty())
                .then(|| format!("<{generic_names_and_constraints}>",))
                .unwrap_or_default(),
            decs
        )?;

        {
            let mut w = IndentWriter::new("\t", &mut *w);
            let w = &mut w;

            for f in &rs.fields {
                self.write_comments(w, &f.comments)?;
                let fixed_name =
                    remove_dash_from_identifier(&swift_keyword_aware_rename(f.id.renamed.as_str()));

                // Create coding keys for serialization / deserialization
                //
                // As of right now this was only written to handle fields
                // that get renamed to an ident with - in it
                if f.id.renamed.as_str().contains("-") {
                    coding_keys.push(format!(r##"{} = "{}""##, fixed_name, &f.id.renamed));

                    // We only need to write out coding keys if we encounter a
                    // situation like this
                    should_write_coding_keys = true;
                } else {
                    coding_keys.push(fixed_name.clone());
                }

                let ty = match f.decorators.type_override_for_lang("swift") {
                    Some(ty) => ty.to_owned(),
                    None => self
                        .format_type(&f.ty, &rs.generic_types)
                        .with_context(|| {
                            format!("failed to format type for field '{}'", f.id.original)
                        })?,
                };

                writeln!(
                    w,
                    "public let {fixed_name}: {ty}{}",
                    (f.has_default && !f.ty.is_optional())
                        .then_some("?")
                        .unwrap_or_default()
                )?;
            }

            if should_write_coding_keys {
                writeln!(
                    w,
                    "\n\
                     enum CodingKeys: String, CodingKey, Codable {{\n\
                     \tcase {keys}\n\
                     }}",
                    keys = coding_keys.join_with(",\n\t\t"),
                )?;
            }

            if !rs.fields.is_empty() {
                writeln!(w)?;
            }

            let mut init_params: Vec<String> = Vec::new();
            for f in &rs.fields {
                let ty = match f.decorators.type_override_for_lang("swift") {
                    Some(ty) => ty.to_owned(),
                    None => self
                        .format_type(&f.ty, &rs.generic_types)
                        .with_context(|| {
                            format!("failed to format type for field '{}'", f.id.original)
                        })?,
                };

                init_params.push(format!(
                    "{}: {}{}",
                    remove_dash_from_identifier(f.id.renamed.as_str()),
                    ty,
                    (f.has_default && !f.ty.is_optional())
                        .then_some("?")
                        .unwrap_or_default()
                ));
            }

            write!(w, "public init({}) {{", init_params.join(", "))?;

            {
                let mut w = IndentWriter::new("\t", &mut *w);
                let w = &mut w;

                if !rs.fields.is_empty() {
                    writeln!(w)?;
                }

                for f in &rs.fields {
                    writeln!(
                        w,
                        "self.{} = {}",
                        remove_dash_from_identifier(f.id.renamed.as_str()),
                        remove_dash_from_identifier(&swift_keyword_aware_rename(
                            f.id.renamed.as_str()
                        ))
                    )?;
                }
            }

            writeln!(w, "}}")?;
        }

        writeln!(w, "}}")?;

        Ok(())
    }

    fn write_enum(&self, w: &mut impl io::Write, e: &RustEnum) -> anyhow::Result<()> {
        let shared = e.shared();
        let enum_name = format!("{}{}", self.prefix, shared.id.original);
        let enum_name = swift_keyword_aware_rename(&enum_name);

        let decs = self
            .get_complete_decorators_for_type(
                match e {
                    RustEnum::Unit { .. } => &["String"],
                    RustEnum::Algebraic { .. } => &[],
                },
                &shared.decorators,
            )
            .join(", ");

        // Make a suitable name for an anonymous struct enum variant
        let make_anonymous_struct_name =
            |variant_name: &TypeName| format!("{}{}Inner", shared.id.renamed, variant_name);

        writeln!(w)?;

        // Generate named types for any anonymous struct variants of this enum
        self.write_struct_types_for_enum_variants(w, e, &make_anonymous_struct_name)?;

        self.write_comments(w, &shared.comments)?;
        let indirect = if shared.is_recursive { "indirect " } else { "" };

        let generic_names_and_constraints =
            self.generic_constraints(&e.shared().decorators, &e.shared().generic_types);

        writeln!(
            w,
            "public {indirect}enum {enum_name}{}: {} {{",
            (!e.shared().generic_types.is_empty())
                .then(|| format!("<{generic_names_and_constraints}>",))
                .unwrap_or_default(),
            decs
        )?;

        let coding_keys_info = self
            .write_enum_variants(w, e, make_anonymous_struct_name)
            .context("failed to write enum variants")?;

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

        writeln!(w, "}}")?;

        Ok(())
    }

    fn write_const(&self, _w: &mut impl io::Write, _c: &RustConst) -> anyhow::Result<()> {
        anyhow::bail!("consts aren't supported by swift yet")
    }

    fn end_file(&self, w: &mut impl io::Write, mode: FilesMode<&CrateName>) -> anyhow::Result<()> {
        if matches!(mode, FilesMode::Single)
            // Relaxed is sufficient: this was set either by this thread, or
            // by a different thread that was joined.
            && self.should_emit_codable_void.load(Ordering::Relaxed)
        {
            self.write_codable_void(w)?;
        }

        Ok(())
    }

    fn write_additional_files<'a>(
        &self,
        output_folder: &Path,
        _output_files: impl IntoIterator<Item = (&'a CrateName, &'a Path)>,
    ) -> anyhow::Result<()> {
        // We could scan all the output files for the presence of CodableVoid.
        // Seems kind of lame, so we'll just use the atomic for now.
        if self.should_emit_codable_void.load(Ordering::Relaxed) {
            let mut content = Vec::new();
            self.write_codable_void(&mut content)
                .expect("write to vec is infallbile");

            let path = output_folder.join("Codable.swift");

            if let Ok(old_content) = fs::read(&path) {
                if content == old_content {
                    return Ok(());
                }
            }

            let mut w = fs::File::create(&path)?;
            w.write_all(&content)?;
        }

        Ok(())
    }
}

fn to_pascal_case(value: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize = true;
    let to_lowercase = {
        // Check if string is all uppercase, such as "URL" or "TOTP". In that case, we don't want
        // to preserve the cases.
        // NOTE: this test should pass if there are letters here, which is
        // why we do the inverted lowercase test
        value.chars().all(|c| !c.is_lowercase())
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

fn to_camel_case(value: &str) -> String {
    let pascal = to_pascal_case(value);
    let mut chars = pascal.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_lowercase().chain(chars).collect()
}

fn swift_keyword_aware_rename(name: &str) -> Cow<'_, str> {
    if SWIFT_KEYWORDS.contains(&name) {
        Cow::Owned(format!("`{name}`"))
    } else {
        Cow::Borrowed(name)
    }
}

fn remove_dash_from_identifier(name: &str) -> String {
    // Dashes are not valid in identifiers, so we map them to underscores
    name.replace('-', "_")
}

#[cfg(test)]
mod rename {
    use super::*;

    macro_rules! cases {
        ($(
            $name:ident : $original:literal => $pascal:literal, $camel:literal;
        )+) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(to_pascal_case($original), $pascal);
                    assert_eq!(to_camel_case($original), $camel);
                }
            )+
        };
    }

    cases! {
        basic: "hello_world" => "HelloWorld", "helloWorld";
        all_lower: "amogus" => "Amogus", "amogus";
    }

    mod op_crypto_regression {
        use super::*;

        cases! {
            rsaoaep: "RSAOAEP" => "Rsaoaep", "rsaoaep";
            rsaoaep256: "RSAOAEP256" => "Rsaoaep256", "rsaoaep256";
        }
    }
}
