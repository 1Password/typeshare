use crate::parser::ParsedData;
use crate::rust_types::{RustEnumShared, RustItem, RustType, RustTypeFormatError, SpecialRustType};
use crate::topsort::topsort;
use crate::RenameExt;
use crate::{
    language::Language,
    rust_types::{
        RustConst, RustConstExpr, RustEnum, RustEnumVariant, RustField, RustStruct, RustTypeAlias,
    },
};
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::OnceLock;
use std::{collections::HashMap, io::Write};

use super::CrateTypes;

use convert_case::{Case, Casing};
use itertools::Itertools;

// Utility function from the original author of supporting Python
// Since we won't be supporting generics right now, this function is unused and is left here for future reference
// Collect unique type vars from an enum field
// Since we explode enums into unions of types, we need to extract all of the generics
// used by each individual field
// We do this by exploring each field's type and comparing against the generics used by the enum
// itself
#[allow(dead_code)]
fn collect_generics_for_variant(variant_type: &RustType, generics: &[String]) -> Vec<String> {
    let mut all = vec![];
    match variant_type {
        RustType::Generic { id, parameters } => {
            if generics.contains(id) {
                all.push(id.clone())
            }
            // Recurse into the params for the case of `Foo(HashMap<K, V>)`
            for param in parameters {
                all.extend(collect_generics_for_variant(param, generics))
            }
        }
        RustType::Simple { id } => {
            if generics.contains(id) {
                all.push(id.clone())
            }
        }
        RustType::Special(special) => match &special {
            SpecialRustType::HashMap(key_type, value_type) => {
                all.extend(collect_generics_for_variant(key_type, generics));
                all.extend(collect_generics_for_variant(value_type, generics));
            }
            SpecialRustType::Option(some_type) => {
                all.extend(collect_generics_for_variant(some_type, generics));
            }
            SpecialRustType::Vec(value_type) => {
                all.extend(collect_generics_for_variant(value_type, generics));
            }
            _ => {}
        },
    }
    // Remove any duplicates
    // E.g. Foo(HashMap<T, T>) should only produce a single type var
    dedup(&mut all);
    all
}

fn dedup<T: Eq + Hash + Clone>(v: &mut Vec<T>) {
    // note the Copy constraint
    let mut uniques = HashSet::new();
    v.retain(|e| uniques.insert(e.clone()));
}

#[derive(Clone)]
struct CustomJsonTranslationFunctions {
    serialization_name: String,
    serialization_content: String,
    deserialization_name: String,
    deserialization_content: String,
}

/// All information needed to generate Python type-code
#[derive(Default)]
pub struct Python {
    /// Mappings from Rust type names to Python type names
    pub type_mappings: HashMap<String, String>,
    /// HashMap<ModuleName, HashSet<Identifier>
    pub imports: HashMap<String, HashSet<String>>,
    /// HashMap<Identifier, Vec<DependencyIdentifiers>>
    /// Used to lay out runtime references in the module
    /// such that it can be read top to bottom
    /// globals: HashMap<String, Vec<String>>,
    pub type_variables: HashSet<String>,
    /// Whether or not to exclude the version header that normally appears at the top of generated code.
    /// If you aren't generating a snapshot test, this setting can just be left as a default (false)
    pub no_version_header: bool,
    /// Carries the unique set of types for custom json translation
    pub types_for_custom_json_translation: HashSet<String>,
}

impl Language for Python {
    fn type_map(&mut self) -> &HashMap<String, String> {
        &self.type_mappings
    }

    fn generate_types(
        &mut self,
        w: &mut dyn Write,
        _imports: &CrateTypes,
        data: ParsedData,
    ) -> std::io::Result<()> {
        self.begin_file(w, &data)?;

        let ParsedData {
            structs,
            enums,
            aliases,
            consts,
            ..
        } = data;

        let mut items = aliases
            .into_iter()
            .map(RustItem::Alias)
            .chain(structs.into_iter().map(RustItem::Struct))
            .chain(enums.into_iter().map(RustItem::Enum))
            .chain(consts.into_iter().map(RustItem::Const))
            .collect::<Vec<_>>();

        topsort(&mut items);

        let mut body: Vec<u8> = Vec::new();
        for thing in items {
            match thing {
                RustItem::Enum(e) => self.write_enum(&mut body, &e)?,
                RustItem::Struct(rs) => self.write_struct(&mut body, &rs)?,
                RustItem::Alias(t) => self.write_type_alias(&mut body, &t)?,
                RustItem::Const(c) => self.write_const(&mut body, &c)?,
            };
        }

        self.write_all_imports(w)?;

        self.types_for_custom_json_translation
            .iter()
            .sorted()
            .filter_map(|py_type| json_translation_for_type(py_type))
            .map(|custom_translation_functions| {
                format!(
                    r#"{}

{}"#,
                    custom_translation_functions.serialization_content,
                    custom_translation_functions.deserialization_content
                )
            })
            .try_for_each(|custom_translation_function| -> std::io::Result<()> {
                writeln!(w, "{custom_translation_function}")?;
                writeln!(w)
            })?;

        w.write_all(&body)
    }

    fn format_generic_type(
        &mut self,
        base: &String,
        parameters: &[RustType],
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        self.add_imports(base);
        if let Some(mapped) = self.type_map().get(base) {
            Ok(mapped.into())
        } else {
            let parameters: Result<Vec<String>, RustTypeFormatError> = parameters
                .iter()
                .map(|p| self.format_type(p, generic_types))
                .collect();
            let parameters = parameters?;
            Ok(format!(
                "{}{}",
                self.format_simple_type(base, generic_types)?,
                if !parameters.is_empty() {
                    format!("[{}]", parameters.join(", "))
                } else {
                    Default::default()
                }
            ))
        }
    }

    fn format_simple_type(
        &mut self,
        base: &String,
        _generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        self.add_imports(base);
        Ok(if let Some(mapped) = self.type_map().get(base) {
            mapped.into()
        } else {
            base.into()
        })
    }

    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
        if let Some(mapped) = self.type_mappings.get(&special_ty.to_string()) {
            if json_translation_for_type(mapped).is_some() {
                self.types_for_custom_json_translation
                    .insert(mapped.to_string());
            }
            return Ok(mapped.to_owned());
        }
        match special_ty {
            SpecialRustType::Array(rtype, _)
            | SpecialRustType::Slice(rtype)
            | SpecialRustType::Vec(rtype) => {
                self.add_import("typing".to_string(), "List".to_string());
                Ok(format!("List[{}]", self.format_type(rtype, generic_types)?))
            }
            // We add optionality above the type formatting level
            SpecialRustType::Option(rtype) => {
                self.add_import("typing".to_string(), "Optional".to_string());
                Ok(format!(
                    "Optional[{}]",
                    self.format_type(rtype, generic_types)?
                ))
            }
            SpecialRustType::HashMap(rtype1, rtype2) => {
                self.add_import("typing".to_string(), "Dict".to_string());
                Ok(format!(
                    "Dict[{}, {}]",
                    match rtype1.as_ref() {
                        RustType::Simple { id } if generic_types.contains(id) => {
                            return Err(RustTypeFormatError::GenericKeyForbiddenInTS(id.clone()));
                        }
                        _ => self.format_type(rtype1, generic_types)?,
                    },
                    self.format_type(rtype2, generic_types)?
                ))
            }
            SpecialRustType::DateTime => {
                self.add_import("datetime".to_string(), "datetime".to_string());
                Ok("datetime".into())
            }
            SpecialRustType::Unit => Ok("None".into()),
            SpecialRustType::String | SpecialRustType::Char => Ok("str".into()),
            SpecialRustType::I8
            | SpecialRustType::U8
            | SpecialRustType::I16
            | SpecialRustType::U16
            | SpecialRustType::I32
            | SpecialRustType::U32
            | SpecialRustType::I54
            | SpecialRustType::U53
            | SpecialRustType::U64
            | SpecialRustType::I64
            | SpecialRustType::U128
            | SpecialRustType::ISize
            | SpecialRustType::USize => Ok("int".into()),
            SpecialRustType::F32 | SpecialRustType::F64 => Ok("float".into()),
            SpecialRustType::Bool => Ok("bool".into()),
        }
    }

    fn begin_file(&mut self, w: &mut dyn Write, _parsed_data: &ParsedData) -> std::io::Result<()> {
        if !self.no_version_header {
            writeln!(w, "\"\"\"")?;
            writeln!(w, " Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
            writeln!(w, "\"\"\"")?;
        }
        Ok(())
    }

    fn write_type_alias(&mut self, w: &mut dyn Write, ty: &RustTypeAlias) -> std::io::Result<()> {
        let r#type = self
            .format_type(&ty.r#type, ty.generic_types.as_slice())
            .map_err(std::io::Error::other)?;

        writeln!(
            w,
            "{}{} = {}\n",
            ty.id.renamed,
            if !ty.generic_types.is_empty() {
                format!("[{}]", ty.generic_types.join(", "))
            } else {
                Default::default()
            },
            r#type,
        )?;

        self.write_comments(w, true, &ty.comments, 0)?;

        Ok(())
    }

    fn write_const(&mut self, w: &mut dyn Write, c: &RustConst) -> std::io::Result<()> {
        match c.expr {
            RustConstExpr::Int(val) => {
                let const_type = self
                    .format_type(&c.r#type, &[])
                    .map_err(std::io::Error::other)?;
                writeln!(
                    w,
                    "{}: {} = {}",
                    c.id.renamed.to_snake_case().to_uppercase(),
                    const_type,
                    val
                )
            }
        }
    }

    fn write_struct(&mut self, w: &mut dyn Write, rs: &RustStruct) -> std::io::Result<()> {
        self.add_import("pydantic".to_string(), "BaseModel".to_string());
        {
            rs.generic_types
                .iter()
                .cloned()
                .for_each(|v| self.add_type_var(v))
        }
        let bases = match rs.generic_types.is_empty() {
            true => "BaseModel".to_string(),
            false => {
                self.add_import("typing".to_string(), "Generic".to_string());
                format!("BaseModel, Generic[{}]", rs.generic_types.join(", "))
            }
        };
        writeln!(w, "class {}({}):", rs.id.renamed, bases,)?;

        self.write_comments(w, true, &rs.comments, 1)?;

        handle_model_config(w, self, &rs.fields);

        rs.fields
            .iter()
            .try_for_each(|f| self.write_field(w, f, rs.generic_types.as_slice()))?;

        if rs.fields.is_empty() {
            write!(w, "    pass")?
        }
        writeln!(w)
    }

    fn write_enum(&mut self, w: &mut dyn Write, e: &RustEnum) -> std::io::Result<()> {
        // Make a suitable name for an anonymous struct enum variant
        let make_anonymous_struct_name =
            |variant_name: &str| format!("{}{}Inner", &e.shared().id.renamed, variant_name);

        // Generate named types for any anonymous struct variants of this enum
        self.write_types_for_anonymous_structs(w, e, &make_anonymous_struct_name)?;
        match e {
            // Write all the unit variants out (there can only be unit variants in
            // this case)
            RustEnum::Unit(shared) => {
                self.add_import("enum".to_string(), "Enum".to_string());
                writeln!(w, "class {}(str, Enum):", shared.id.renamed)?;
                // let comment = shared.comments.join("\n");
                self.write_comments(w, true, &shared.comments, 1)?;
                if shared.variants.is_empty() {
                    writeln!(w, "    pass")?;
                } else {
                    shared.variants.iter().try_for_each(|v| {
                        writeln!(
                            w,
                            "    {} = \"{}\"",
                            v.shared().id.original.to_uppercase(),
                            match v {
                                RustEnumVariant::Unit(v) => {
                                    v.id.renamed.replace("\"", "\\\"")
                                }
                                _ => unreachable!("Only unit variants are allowed here"),
                            }
                        )?;
                        self.write_comments(w, true, &v.shared().comments, 1)
                    })?
                };
            }
            // Write all the algebraic variants out (all three variant types are possible
            // here)
            RustEnum::Algebraic {
                tag_key,
                content_key,
                shared,
                ..
            } => {
                self.write_algebraic_enum(
                    tag_key,
                    content_key,
                    &e.shared().id.renamed,
                    shared,
                    w,
                    &make_anonymous_struct_name,
                )?;
            }
        };
        Ok(())
    }

    fn write_imports(
        &mut self,
        _writer: &mut dyn Write,
        _imports: super::ScopedCrateTypes<'_>,
    ) -> std::io::Result<()> {
        // TODO: to implement when adding suport for outputting to multiple files.
        Ok(())
    }
}

impl Python {
    fn add_imports(&mut self, tp: &str) {
        match tp {
            "Url" => {
                self.add_import("pydantic.networks".to_string(), "AnyUrl".to_string());
            }
            "DateTime" => {
                self.add_import("datetime".to_string(), "datetime".to_string());
            }
            _ => {}
        }
    }

    fn add_common_imports(
        &mut self,
        is_optional: bool,
        requires_custom_translation: bool,
        is_aliased: bool,
    ) {
        if is_optional {
            self.add_import("typing".to_string(), "Optional".to_string());
        }
        if requires_custom_translation {
            self.add_import("pydantic".to_string(), "BeforeValidator".to_string());
            self.add_import("pydantic".to_string(), "PlainSerializer".to_string());
            self.add_import("typing".to_string(), "Annotated".to_string());
        }
        if is_aliased || is_optional {
            self.add_import("pydantic".to_string(), "Field".to_string());
        }
    }

    fn write_field(
        &mut self,
        w: &mut dyn Write,
        field: &RustField,
        generic_types: &[String],
    ) -> std::io::Result<()> {
        let is_optional = field.ty.is_optional() || field.has_default;
        // currently, if a field has a serde default value, it must be an Option
        let not_optional_but_default = !field.ty.is_optional() && field.has_default;
        let python_type = self
            .format_type(&field.ty, generic_types)
            .map_err(std::io::Error::other)?;
        let python_field_name = python_property_aware_rename(&field.id.original);
        let is_aliased = python_field_name != field.id.renamed;
        let custom_translations = json_translation_for_type(&python_type);
        // Adds all the required imports needed based off whether its optional ,aliased, or needs a byte translation
        self.add_common_imports(is_optional, custom_translations.is_some(), is_aliased);

        let mut field_type = python_type;

        if not_optional_but_default {
            field_type = format!("Optional[{field_type}]");
        }
        if let Some(custom_translation) = custom_translations {
            self.types_for_custom_json_translation
                .insert(field_type.clone());
            field_type = format!(
                "Annotated[{field_type}, BeforeValidator({}), PlainSerializer({})]",
                custom_translation.deserialization_name, custom_translation.serialization_name
            );
        }

        let mut decorators: Vec<String> = Vec::new();
        if is_aliased {
            decorators.push(format!("alias=\"{}\"", field.id.renamed));
        }

        if is_optional || not_optional_but_default {
            decorators.push("default=None".to_string());
        }

        let python_return_value = if !decorators.is_empty() {
            format!(" = Field({})", decorators.join(", "))
        } else {
            String::new()
        };

        writeln!(
            w,
            r#"    {python_field_name}: {field_type}{python_return_value}"#
        )?;

        self.write_comments(w, true, &field.comments, 1)?;
        Ok(())
    }

    fn write_comments(
        &self,
        w: &mut dyn Write,
        is_docstring: bool,
        comments: &[String],
        indent_level: usize,
    ) -> std::io::Result<()> {
        // Only attempt to write a comment if there are some, otherwise we're Ok()
        let indent = "    ".repeat(indent_level);
        if !comments.is_empty() {
            let comment: String = {
                if is_docstring {
                    format!(
                        "{indent}\"\"\"\n{indented_comments}\n{indent}\"\"\"",
                        indent = indent,
                        indented_comments = comments
                            .iter()
                            .map(|v| format!("{indent}{v}"))
                            .collect::<Vec<String>>()
                            .join("\n"),
                    )
                } else {
                    comments
                        .iter()
                        .map(|v| format!("{indent}# {v}"))
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            };
            writeln!(w, "{comment}")?;
        }
        Ok(())
    }

    // Idempotently insert an import
    fn add_import(&mut self, module: String, identifier: String) {
        self.imports.entry(module).or_default().insert(identifier);
    }

    fn add_type_var(&mut self, name: String) {
        self.add_import("typing".to_string(), "TypeVar".to_string());
        self.type_variables.insert(name);
    }

    fn write_all_imports(&self, w: &mut dyn Write) -> std::io::Result<()> {
        let mut type_var_names: Vec<String> = self.type_variables.iter().cloned().collect();
        type_var_names.sort();
        let type_vars: Vec<String> = type_var_names
            .iter()
            .map(|name| format!("{name} = TypeVar(\"{name}\")"))
            .collect();
        let mut imports = vec![];
        for (import_module, identifiers) in &self.imports {
            let mut identifier_vec = identifiers.iter().cloned().collect::<Vec<String>>();
            identifier_vec.sort();
            imports.push(format!(
                "from {} import {}",
                import_module,
                identifier_vec.join(", ")
            ))
        }
        imports.sort();

        writeln!(w, "from __future__ import annotations\n")?;
        writeln!(w, "{}\n", imports.join("\n"))?;

        match type_vars.is_empty() {
            true => writeln!(w)?,
            false => writeln!(w, "{}\n\n", type_vars.join("\n"))?,
        };
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn write_variant_class(
        &mut self,
        class_name: &str,
        tag_key: &str,
        tag_value: &str,
        content_key: &str,
        content_type: Option<&str>,
        content_value: Option<&str>,
        comments: &[String],
        w: &mut dyn Write,
    ) -> std::io::Result<()> {
        self.add_import("typing".to_string(), "Literal".to_string());
        writeln!(w, "class {class_name}(BaseModel):")?;
        self.write_comments(w, true, comments, 1)?;

        writeln!(w, "    {tag_key}: Literal[{tag_value}] = {tag_value}",)?;
        if content_type.is_none() && content_value.is_none() {
            return Ok(());
        }
        writeln!(
            w,
            "    {content_key}{}{}",
            if let Some(content_type) = content_type {
                format!(": {content_type}")
            } else {
                "".to_string()
            },
            if let Some(content_value) = content_value {
                format!(" = {content_value}")
            } else {
                "".to_string()
            }
        )?;
        Ok(())
    }
    fn write_algebraic_enum(
        &mut self,
        tag_key: &str,
        content_key: &str,
        enum_name: &str,
        shared: &RustEnumShared,
        w: &mut dyn Write,
        make_struct_name: &dyn Fn(&str) -> String,
    ) -> std::io::Result<()> {
        shared
            .generic_types
            .iter()
            .cloned()
            .for_each(|v| self.add_type_var(v));
        self.add_import("pydantic".to_string(), "BaseModel".to_string());
        // all the types and class names for the enum variants in tuple
        // (type_name, class_name)
        let all_enum_variants_name = shared
            .variants
            .iter()
            .map(|v| match v {
                RustEnumVariant::Unit(v) => v.id.renamed.clone(),
                RustEnumVariant::Tuple { shared, .. } => shared.id.renamed.clone(),
                RustEnumVariant::AnonymousStruct { shared, .. } => shared.id.renamed.clone(),
            })
            .map(|name| (name.to_case(Case::Snake).to_uppercase(), name))
            .collect::<Vec<(String, String)>>();
        let enum_type_class_name = format!("{}Types", shared.id.renamed);
        self.add_import("enum".to_string(), "Enum".to_string());
        // write "types" class: a union of all the enum variants
        writeln!(w, "class {enum_type_class_name}(str, Enum):")?;
        writeln!(
            w,
            "{}",
            all_enum_variants_name
                .iter()
                .map(|(type_key_name, type_string)| format!(
                    "    {type_key_name} = \"{type_string}\""
                ))
                .collect::<Vec<String>>()
                .join("\n")
        )?;
        writeln!(w)?;

        let mut union_members = Vec::new();
        // write each of the enum variant as a class:
        for (variant, (type_key_name, ..)) in
            shared.variants.iter().zip(all_enum_variants_name.iter())
        {
            let variant_class_name = format!("{enum_name}{}", &variant.shared().id.original);
            union_members.push(variant_class_name.clone());
            match variant {
                RustEnumVariant::Unit(variant_shared) => {
                    self.write_variant_class(
                        &variant_class_name,
                        tag_key,
                        format!("{enum_type_class_name}.{type_key_name}",).as_str(),
                        content_key,
                        None,
                        None,
                        &variant_shared.comments,
                        w,
                    )?;
                    writeln!(w)?;
                }
                RustEnumVariant::Tuple {
                    ty,
                    shared: variant_shared,
                } => {
                    let tuple_name = self
                        .format_type(ty, shared.generic_types.as_slice())
                        .map_err(std::io::Error::other)?;
                    self.write_variant_class(
                        &variant_class_name,
                        tag_key,
                        format!("{enum_type_class_name}.{type_key_name}",).as_str(),
                        content_key,
                        Some(&tuple_name),
                        None,
                        &variant_shared.comments,
                        w,
                    )?;
                    writeln!(w)?;
                }
                RustEnumVariant::AnonymousStruct {
                    shared: variant_shared,
                    ..
                } => {
                    // writing is taken care of by write_types_for_anonymous_structs in write_enum
                    let variant_class_inner_name = make_struct_name(&variant_shared.id.original);

                    self.write_variant_class(
                        &variant_class_name,
                        tag_key,
                        format!("{enum_type_class_name}.{type_key_name}",).as_str(),
                        content_key,
                        Some(&variant_class_inner_name),
                        None,
                        &variant_shared.comments,
                        w,
                    )?;
                    writeln!(w)?;
                }
            }
        }

        self.write_comments(w, false, &shared.comments, 0)?;
        if union_members.len() == 1 {
            writeln!(w, "{enum_name} = {}", union_members[0])?;
        } else {
            self.add_import("typing".to_string(), "Union".to_string());
            writeln!(w, "{enum_name} = Union[{}]", union_members.join(", "))?;
        }
        Ok(())
    }
}

static PYTHON_KEYWORDS: OnceLock<HashSet<String>> = OnceLock::new();

fn get_python_keywords() -> &'static HashSet<String> {
    PYTHON_KEYWORDS.get_or_init(|| {
        HashSet::from_iter(
            vec![
                "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
                "continue", "def", "del", "elif", "else", "except", "finally", "for", "from",
                "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass",
                "raise", "return", "try", "while", "with", "yield",
            ]
            .iter()
            .map(|v| v.to_string()),
        )
    })
}

fn python_property_aware_rename(name: &str) -> String {
    let snake_name = name.to_case(Case::Snake);
    match get_python_keywords().contains(&snake_name) {
        true => format!("{name}_"),
        false => snake_name,
    }
}

// If at least one field from within a class is changed when the serde rename is used (a.k.a the field has 2 words) then we must use aliasing and we must also use a config dict at the top level of the class.
fn handle_model_config(w: &mut dyn Write, python_module: &mut Python, fields: &[RustField]) {
    let visibly_renamed_field = fields.iter().find(|f| {
        let python_field_name = python_property_aware_rename(&f.id.original);
        python_field_name != f.id.renamed
    });
    if visibly_renamed_field.is_some() {
        python_module.add_import("pydantic".to_string(), "ConfigDict".to_string());
        let _ = writeln!(w, "    model_config = ConfigDict(populate_by_name=True)\n");
    };
}

/// acquires custom translation function names if custom serialize/deserialize functions are needed
fn json_translation_for_type(python_type: &str) -> Option<CustomJsonTranslationFunctions> {
    // if more custom serialization/deserialization is needed, we can add it here and in the hashmap below
    let custom_translations = HashMap::from([
        (
            "bytes",
            CustomJsonTranslationFunctions {
                serialization_name: "serialize_binary_data".to_owned(),
                serialization_content: r#"def serialize_binary_data(value: bytes) -> list[int]:
        return list(value)"#
                    .to_owned(),
                deserialization_name: "deserialize_binary_data".to_owned(),
                deserialization_content: r#"def deserialize_binary_data(value):
     if isinstance(value, list):
         if all(isinstance(x, int) and 0 <= x <= 255 for x in value):
            return bytes(value)
         raise ValueError("All elements must be integers in the range 0-255 (u8).")
     elif isinstance(value, bytes):
            return value
     raise TypeError("Content must be a list of integers (0-255) or bytes.")"#
                    .to_owned(),
            },
        ),
        (
            "datetime",
            CustomJsonTranslationFunctions {
                serialization_name: "serialize_datetime_data".to_owned(),
                serialization_content: r#"def serialize_datetime_data(utc_time: datetime) -> str:
        return utc_time.strftime("%Y-%m-%dT%H:%M:%S.%fZ")"#
                    .to_owned(),
                deserialization_name: "parse_rfc3339".to_owned(),
                deserialization_content: r#"def parse_rfc3339(date_str: str) -> datetime:
    date_formats = [
        "%Y-%m-%dT%H:%M:%SZ",   
        "%Y-%m-%dT%H:%M:%S.%fZ"
    ]
    
    for fmt in date_formats:
        try:
            return datetime.strptime(date_str, fmt)
        except ValueError:
            continue
    
    raise ValueError(f"Invalid RFC 3339 date format: {date_str}")"#
                    .to_owned(),
            },
        ),
    ]);

    custom_translations
        .get(python_type)
        .map(|custom_translation| (*custom_translation).to_owned())
}

#[cfg(test)]
mod test {
    use crate::rust_types::Id;

    use super::*;
    #[test]
    fn test_python_property_aware_rename() {
        assert_eq!(python_property_aware_rename("class"), "class_");
        assert_eq!(python_property_aware_rename("snake_case"), "snake_case");
    }

    #[test]
    fn test_optional_value_with_serde_default() {
        let mut python = Python::default();
        let mock_writer = &mut Vec::new();
        let rust_field = RustField {
            id: Id {
                original: "field".to_string(),
                renamed: "field".to_string(),
                serde_rename: false,
            },
            ty: RustType::Special(SpecialRustType::Option(Box::new(RustType::Simple {
                id: "str".to_string(),
            }))),
            has_default: true,
            comments: Default::default(),
            decorators: Default::default(),
        };
        python.write_field(mock_writer, &rust_field, &[]).unwrap();
        assert_eq!(
            String::from_utf8_lossy(mock_writer),
            "    field: Optional[str] = Field(default=None)\n"
        );
    }

    #[test]
    fn test_optional_value_no_serde_default() {
        let mut python = Python::default();
        let mock_writer = &mut Vec::new();
        let rust_field = RustField {
            id: Id {
                original: "field".to_string(),
                renamed: "field".to_string(),
                serde_rename: false,
            },
            ty: RustType::Special(SpecialRustType::Option(Box::new(RustType::Simple {
                id: "str".to_string(),
            }))),
            has_default: false,
            comments: Default::default(),
            decorators: Default::default(),
        };
        python.write_field(mock_writer, &rust_field, &[]).unwrap();
        assert_eq!(
            String::from_utf8_lossy(mock_writer),
            "    field: Optional[str] = Field(default=None)\n"
        );
    }

    #[test]
    fn test_non_optional_value_with_serde_default() {
        // technically an invalid case at the moment, as we don't support serde default values other than None
        // TODO: change this test if we do
        let mut python = Python::default();
        let mock_writer = &mut Vec::new();
        let rust_field = RustField {
            id: Id {
                original: "field".to_string(),
                renamed: "field".to_string(),
                serde_rename: false,
            },
            ty: RustType::Simple {
                id: "str".to_string(),
            },
            has_default: true,
            comments: Default::default(),
            decorators: Default::default(),
        };
        python.write_field(mock_writer, &rust_field, &[]).unwrap();
        assert_eq!(
            String::from_utf8_lossy(mock_writer),
            "    field: Optional[str] = Field(default=None)\n"
        );
    }

    #[test]
    fn test_non_optional_value_with_no_serde_default() {
        let mut python = Python::default();
        let mock_writer = &mut Vec::new();
        let rust_field = RustField {
            id: Id {
                original: "field".to_string(),
                renamed: "field".to_string(),
                serde_rename: false,
            },
            ty: RustType::Simple {
                id: "str".to_string(),
            },
            has_default: false,
            comments: Default::default(),
            decorators: Default::default(),
        };
        python.write_field(mock_writer, &rust_field, &[]).unwrap();
        assert_eq!(String::from_utf8_lossy(mock_writer), "    field: str\n");
    }
}
