use crate::parser::ParsedData;
use crate::rust_types::{RustType, RustTypeFormatError, SpecialRustType};
use crate::{
    language::Language,
    rust_types::{RustEnum, RustEnumVariant, RustField, RustStruct, RustTypeAlias},
};
use once_cell::sync::Lazy;
use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::hash::Hash;
use std::{collections::HashMap, io::Write};

use super::CrateTypes;

use convert_case::{Case, Casing};
use topological_sort::TopologicalSort;

#[derive(Debug, Default)]
pub struct Module {
    // HashMap<ModuleName, HashSet<Identifier>
    imports: HashMap<String, HashSet<String>>,
    // HashMap<Identifier, Vec<DependencyIdentifiers>>
    // Used to lay out runtime references in the module
    // such that it can be read top to bottom
    globals: HashMap<String, Vec<String>>,
    type_variables: HashSet<String>,
}

#[derive(Debug)]
struct GenerationError;

impl Module {
    // Idempotently insert an import
    fn add_import(&mut self, module: String, identifier: String) {
        self.imports.entry(module).or_default().insert(identifier);
    }
    fn add_global(&mut self, identifier: String, deps: Vec<String>) {
        match self.globals.entry(identifier) {
            Entry::Occupied(mut e) => e.get_mut().extend_from_slice(&deps),
            Entry::Vacant(e) => {
                e.insert(deps);
            }
        }
    }
    fn add_type_var(&mut self, name: String) {
        self.add_import("typing".to_string(), "TypeVar".to_string());
        self.type_variables.insert(name);
    }
    fn get_type_vars(&mut self, n: usize) -> Vec<String> {
        let vars: Vec<String> = (0..n)
            .map(|i| {
                if i == 0 {
                    "T".to_string()
                } else {
                    format!("T{}", i)
                }
            })
            .collect();
        vars.iter().for_each(|tv| self.add_type_var(tv.clone()));
        vars
    }
    // Rust lets you declare type aliases before the struct they point to.
    // But in Python we need the struct to come first.
    // So we need to topologically sort the globals so that type aliases
    // always come _after_ the struct/enum they point to.
    fn topologically_sorted_globals(&self) -> Result<Vec<String>, GenerationError> {
        let mut ts: TopologicalSort<String> = TopologicalSort::new();
        for (identifier, dependencies) in &self.globals {
            for dependency in dependencies {
                ts.add_dependency(dependency.clone(), identifier.clone())
            }
        }
        let mut res: Vec<String> = Vec::new();
        loop {
            let mut level = ts.pop_all();
            level.sort();
            res.extend_from_slice(&level);
            if level.is_empty() {
                if !ts.is_empty() {
                    return Err(GenerationError);
                }
                break;
            }
        }
        let existing: HashSet<&String> = HashSet::from_iter(res.iter());
        let mut missing: Vec<String> = self
            .globals
            .keys()
            .filter(|&k| !existing.contains(k))
            .cloned()
            .collect();
        missing.sort();
        res.extend(missing);
        Ok(res)
    }
}

#[derive(Debug, Clone)]
enum ParsedRustThing<'a> {
    Struct(&'a RustStruct),
    Enum(&'a RustEnum),
    TypeAlias(&'a RustTypeAlias),
}

// Collect unique type vars from an enum field
// Since we explode enums into unions of types, we need to extract all of the generics
// used by each individual field
// We do this by exploring each field's type and comparing against the generics used by the enum
// itself
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

/// All information needed to generate Python type-code
#[derive(Default)]
pub struct Python {
    /// Mappings from Rust type names to Python type names
    pub type_mappings: HashMap<String, String>,
    /// The Python module for the generated code.
    pub module: Module,
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
        let mut globals: Vec<ParsedRustThing>;
        {
            for alias in &data.aliases {
                let thing = ParsedRustThing::TypeAlias(alias);
                let identifier = self.get_identifier(thing);
                match &alias.r#type {
                    RustType::Generic { id, parameters: _ } => {
                        self.module.add_global(identifier, vec![id.clone()])
                    }
                    RustType::Simple { id } => self.module.add_global(identifier, vec![id.clone()]),
                    RustType::Special(_) => {}
                }
            }
            for strct in &data.structs {
                let thing = ParsedRustThing::Struct(strct);
                let identifier = self.get_identifier(thing);
                self.module.add_global(identifier, vec![]);
            }
            for enm in &data.enums {
                let thing = ParsedRustThing::Enum(enm);
                let identifier = self.get_identifier(thing);
                self.module.add_global(identifier, vec![]);
            }
            globals = data
                .aliases
                .iter()
                .map(ParsedRustThing::TypeAlias)
                .chain(data.structs.iter().map(ParsedRustThing::Struct))
                .chain(data.enums.iter().map(ParsedRustThing::Enum))
                .collect();
            let sorted_identifiers = self.module.topologically_sorted_globals().unwrap();
            globals.sort_by(|a, b| {
                let identifier_a = self.get_identifier(a.clone());
                let identifier_b = self.get_identifier(b.clone());
                let pos_a = sorted_identifiers
                    .iter()
                    .position(|o| o.eq(&identifier_a))
                    .unwrap_or(0);
                let pos_b = sorted_identifiers
                    .iter()
                    .position(|o| o.eq(&identifier_b))
                    .unwrap_or(0);
                pos_a.cmp(&pos_b)
            });
        }
        let mut body: Vec<u8> = Vec::new();
        for thing in globals {
            match thing {
                ParsedRustThing::Enum(e) => self.write_enum(&mut body, e)?,
                ParsedRustThing::Struct(rs) => self.write_struct(&mut body, rs)?,
                ParsedRustThing::TypeAlias(t) => self.write_type_alias(&mut body, t)?,
            };
        }
        self.begin_file(w, &data)?;
        let _ = w.write(&body)?;
        Ok(())
    }

    fn format_generic_type(
        &mut self,
        base: &String,
        parameters: &[RustType],
        generic_types: &[String],
    ) -> Result<String, RustTypeFormatError> {
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
                (!parameters.is_empty())
                    .then(|| format!("[{}]", parameters.join(", ")))
                    .unwrap_or_default()
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
        match special_ty {
            SpecialRustType::Vec(rtype)
            | SpecialRustType::Array(rtype, _)
            | SpecialRustType::Slice(rtype) => {
                self.module
                    .add_import("typing".to_string(), "List".to_string());
                Ok(format!("List[{}]", self.format_type(rtype, generic_types)?))
            }
            // We add optionality above the type formatting level
            SpecialRustType::Option(rtype) => self.format_type(rtype, generic_types),
            SpecialRustType::HashMap(rtype1, rtype2) => {
                self.module
                    .add_import("typing".to_string(), "Dict".to_string());
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
            | SpecialRustType::ISize
            | SpecialRustType::USize => Ok("int".into()),
            SpecialRustType::F32 | SpecialRustType::F64 => Ok("float".into()),
            SpecialRustType::Bool => Ok("bool".into()),
        }
    }

    fn begin_file(&mut self, w: &mut dyn Write, _parsed_data: &ParsedData) -> std::io::Result<()> {
        let mut type_var_names: Vec<String> = self.module.type_variables.iter().cloned().collect();
        type_var_names.sort();
        let type_vars: Vec<String> = type_var_names
            .iter()
            .map(|name| format!("{} = TypeVar(\"{}\")", name, name))
            .collect();
        let mut imports = vec![];
        for (import_module, identifiers) in &self.module.imports {
            let mut identifier_vec = identifiers.iter().cloned().collect::<Vec<String>>();
            identifier_vec.sort();
            imports.push(format!(
                "from {} import {}",
                import_module,
                identifier_vec.join(", ")
            ))
        }
        imports.sort();
        writeln!(w, "\"\"\"")?;
        writeln!(w, " Generated by typeshare {}", env!("CARGO_PKG_VERSION"))?;
        writeln!(w, "\"\"\"")?;
        writeln!(w, "from __future__ import annotations\n").unwrap();
        writeln!(w, "{}\n", imports.join("\n"))?;
        match type_vars.is_empty() {
            true => writeln!(w).unwrap(),
            false => writeln!(w, "{}\n\n", type_vars.join("\n")).unwrap(),
        };
        Ok(())
    }

    fn write_type_alias(&mut self, w: &mut dyn Write, ty: &RustTypeAlias) -> std::io::Result<()> {
        let r#type = self
            .format_type(&ty.r#type, ty.generic_types.as_slice())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        writeln!(
            w,
            "{}{} = {}\n\n",
            ty.id.renamed,
            (!ty.generic_types.is_empty())
                .then(|| format!("[{}]", ty.generic_types.join(", ")))
                .unwrap_or_default(),
            r#type,
        )?;

        self.write_comments(w, true, &ty.comments, 1)?;

        Ok(())
    }

    fn write_struct(&mut self, w: &mut dyn Write, rs: &RustStruct) -> std::io::Result<()> {
        {
            rs.generic_types
                .iter()
                .cloned()
                .for_each(|v| self.module.add_type_var(v))
        }
        let bases = match rs.generic_types.is_empty() {
            true => "BaseModel".to_string(),
            false => {
                self.module
                    .add_import("pydantic.generics".to_string(), "GenericModel".to_string());
                self.module
                    .add_import("typing".to_string(), "Generic".to_string());
                format!("GenericModel, Generic[{}]", rs.generic_types.join(", "))
            }
        };
        writeln!(w, "class {}({}):", rs.id.renamed, bases,)?;

        self.write_comments(w, true, &rs.comments, 1)?;

        handle_model_config(w, &mut self.module, rs);

        rs.fields
            .iter()
            .try_for_each(|f| self.write_field(w, f, rs.generic_types.as_slice()))?;

        if rs.fields.is_empty() {
            write!(w, "    pass")?
        }
        write!(w, "\n\n")?;
        self.module
            .add_import("pydantic".to_string(), "BaseModel".to_string());
        Ok(())
    }

    fn write_enum(&mut self, w: &mut dyn Write, e: &RustEnum) -> std::io::Result<()> {
        // Make a suitable name for an anonymous struct enum variant
        let make_anonymous_struct_name =
            |variant_name: &str| format!("{}{}Inner", &e.shared().id.original, variant_name);

        // Generate named types for any anonymous struct variants of this enum
        self.write_types_for_anonymous_structs(w, e, &make_anonymous_struct_name)?;

        match e {
            // Write all the unit variants out (there can only be unit variants in
            // this case)
            RustEnum::Unit(shared) => {
                self.module
                    .add_import("typing".to_string(), "Literal".to_string());
                write!(
                    w,
                    "{} = Literal[{}]",
                    shared.id.renamed,
                    shared
                        .variants
                        .iter()
                        .map(|v| format!(
                            "\"{}\"",
                            match v {
                                RustEnumVariant::Unit(v) => {
                                    v.id.renamed.clone()
                                }
                                _ => panic!(),
                            }
                        ))
                        .collect::<Vec<String>>()
                        .join(", ")
                )?;
                write!(w, "\n\n").unwrap();
            }
            // Write all the algebraic variants out (all three variant types are possible
            // here)
            RustEnum::Algebraic {
                tag_key,
                content_key,
                shared,
                ..
            } => {
                {
                    shared
                        .generic_types
                        .iter()
                        .cloned()
                        .for_each(|v| self.module.add_type_var(v))
                }
                let mut variants: Vec<(String, Vec<String>)> = Vec::new();
                shared.variants.iter().for_each(|variant| {
                    match variant {
                        RustEnumVariant::Unit(unit_variant) => {
                            self.module
                                .add_import("typing".to_string(), "Literal".to_string());
                            let variant_name =
                                format!("{}{}", shared.id.original, unit_variant.id.original);
                            variants.push((variant_name.clone(), vec![]));
                            writeln!(w, "class {}:", variant_name).unwrap();
                            writeln!(
                                w,
                                "    {}: Literal[\"{}\"]",
                                tag_key, unit_variant.id.renamed
                            )
                            .unwrap();
                        }
                        RustEnumVariant::Tuple {
                            ty,
                            shared: variant_shared,
                        } => {
                            self.module
                                .add_import("typing".to_string(), "Literal".to_string());
                            let variant_name =
                                format!("{}{}", shared.id.original, variant_shared.id.original);
                            match ty {
                                RustType::Generic { id: _, parameters } => {
                                    // This variant has generics, include them in the class def
                                    let mut generic_parameters: Vec<String> = parameters
                                        .iter()
                                        .flat_map(|p| {
                                            collect_generics_for_variant(p, &shared.generic_types)
                                        })
                                        .collect();
                                    dedup(&mut generic_parameters);
                                    let type_vars =
                                        self.module.get_type_vars(generic_parameters.len());
                                    variants.push((variant_name.clone(), type_vars));
                                    {
                                        if generic_parameters.is_empty() {
                                            self.module.add_import(
                                                "pydantic".to_string(),
                                                "BaseModel".to_string(),
                                            );
                                            writeln!(w, "class {}(BaseModel):", variant_name)
                                                .unwrap();
                                        } else {
                                            self.module.add_import(
                                                "typing".to_string(),
                                                "Generic".to_string(),
                                            );
                                            self.module.add_import(
                                                "pydantic.generics".to_string(),
                                                "GenericModel".to_string(),
                                            );
                                            writeln!(
                                                w,
                                                "class {}(GenericModel, Generic[{}]):",
                                                // note: generics is always unique (a single item)
                                                variant_name,
                                                generic_parameters.join(", ")
                                            )
                                            .unwrap();
                                        }
                                    }
                                }
                                other => {
                                    let mut generics = vec![];
                                    if let RustType::Simple { id } = other {
                                        // This could be a bare generic
                                        if shared.generic_types.contains(id) {
                                            generics = vec![id.clone()];
                                        }
                                    }
                                    variants.push((variant_name.clone(), generics.clone()));
                                    {
                                        if generics.is_empty() {
                                            self.module.add_import(
                                                "pydantic".to_string(),
                                                "BaseModel".to_string(),
                                            );
                                            writeln!(w, "class {}(BaseModel):", variant_name)
                                                .unwrap();
                                        } else {
                                            self.module.add_import(
                                                "typing".to_string(),
                                                "Generic".to_string(),
                                            );
                                            self.module.add_import(
                                                "pydantic.generics".to_string(),
                                                "GenericModel".to_string(),
                                            );
                                            writeln!(
                                                w,
                                                "class {}(GenericModel, Generic[{}]):",
                                                // note: generics is always unique (a single item)
                                                variant_name,
                                                generics.join(", ")
                                            )
                                            .unwrap();
                                        }
                                    }
                                }
                            };
                            writeln!(
                                w,
                                "    {}: Literal[\"{}\"]",
                                tag_key, variant_shared.id.renamed
                            )
                            .unwrap();
                            writeln!(
                                w,
                                "    {}: {}",
                                content_key,
                                match ty {
                                    RustType::Simple { id } => id.to_owned(),
                                    RustType::Special(special_ty) => self
                                        .format_special_type(special_ty, &shared.generic_types)
                                        .unwrap(),
                                    RustType::Generic { id, parameters } => {
                                        self.format_generic_type(id, parameters, &[]).unwrap()
                                    }
                                }
                            )
                            .unwrap();
                            write!(w, "\n\n").unwrap();
                        }
                        RustEnumVariant::AnonymousStruct {
                            shared: variant_shared,
                            fields,
                        } => {
                            let num_generic_parameters = fields
                                .iter()
                                .flat_map(|f| {
                                    collect_generics_for_variant(&f.ty, &shared.generic_types)
                                })
                                .count();
                            let type_vars = self.module.get_type_vars(num_generic_parameters);
                            let name = make_anonymous_struct_name(&variant_shared.id.original);
                            variants.push((name, type_vars));
                        }
                    };
                });
                writeln!(
                    w,
                    "{} = {}",
                    shared.id.original,
                    variants
                        .iter()
                        .map(|(name, parameters)| match parameters.is_empty() {
                            true => name.clone(),
                            false => format!("{}[{}]", name, parameters.join(", ")),
                        })
                        .collect::<Vec<String>>()
                        .join(" | ")
                )
                .unwrap();
                self.write_comments(w, true, &e.shared().comments, 0)?;
                writeln!(w).unwrap();
            }
        };
        Ok(())
    }

    fn write_imports(
        &mut self,
        _writer: &mut dyn Write,
        _imports: super::ScopedCrateTypes<'_>,
    ) -> std::io::Result<()> {
        todo!()
    }
}

impl Python {
    fn add_imports(&mut self, tp: &str) {
        match tp {
            "Url" => {
                self.module
                    .add_import("pydantic.networks".to_string(), "AnyUrl".to_string());
            }
            "DateTime" => {
                self.module
                    .add_import("datetime".to_string(), "datetime".to_string());
            }
            _ => {}
        }
    }

    fn get_identifier(&self, thing: ParsedRustThing) -> String {
        match thing {
            ParsedRustThing::TypeAlias(alias) => alias.id.original.clone(),
            ParsedRustThing::Struct(strct) => strct.id.original.clone(),
            ParsedRustThing::Enum(enm) => match enm {
                RustEnum::Unit(u) => u.id.original.clone(),
                RustEnum::Algebraic {
                    tag_key: _,
                    content_key: _,
                    shared,
                } => shared.id.original.clone(),
            },
        }
    }

    fn write_field(
        &mut self,
        w: &mut dyn Write,
        field: &RustField,
        generic_types: &[String],
    ) -> std::io::Result<()> {
        let mut python_type = self
            .format_type(&field.ty, generic_types)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let python_field_name = python_property_aware_rename(&field.id.original);
        python_type = match python_field_name == field.id.renamed{
            true => python_type,
            false => {
                self.module
                    .add_import("typing".to_string(), "Annotated".to_string());
                self.module
                    .add_import("pydantic".to_string(), "Field".to_string());
                format!(
                    "Annotated[{}, Field(alias=\"{}\")] = None",
                    python_type, field.id.renamed
                )
            }
        };
        if field.ty.is_optional() || field.has_default && python_field_name == field.id.renamed {
            python_type = format!("Optional[{}] = None", python_type);
            self.module
                .add_import("typing".to_string(), "Optional".to_string());
        }
        let mut default = None;
        if field.has_default {
            default = Some("None".to_string())
        }
        match default {
            Some(default) => writeln!(
                w,
                "    {}: {} = {}",
                python_field_name, python_type, default,
            )?,
            None => writeln!(w, "    {}: {}", python_field_name, python_type)?,
        }

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
                            .map(|v| format!("{}{}", indent, v))
                            .collect::<Vec<String>>()
                            .join("\n"),
                    )
                } else {
                    comments
                        .iter()
                        .map(|v| format!("{}# {}", indent, v))
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            };
            writeln!(w, "{}", comment)?;
        }
        Ok(())
    }
}

static PYTHON_KEYWORDS: Lazy<HashSet<String>> = Lazy::new(|| {
    HashSet::from_iter(
        vec![
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield",
        ]
        .iter()
        .map(|v| v.to_string()),
    )
});

fn python_property_aware_rename(name: &str) -> String {
    let snake_name = name.to_case(Case::Snake);
    match PYTHON_KEYWORDS.contains(&snake_name) {
        true => format!("{}_", name),
        false => snake_name,
    }
}

// If at least one field from within a class is changed when the serde rename is used (a.k.a the field has 2 words) then we must use aliasing and we must also use a config dict at the top level of the class.
fn handle_model_config(w: &mut dyn Write, python_module: &mut Module, rs: &RustStruct) {
    let visibly_renamed_field = rs.fields.iter().find(|f| {
        let python_field_name = python_property_aware_rename(&f.id.original);
        python_field_name != f.id.renamed
    });
    if visibly_renamed_field.is_some() {
        python_module.add_import("pydantic".to_string(), "ConfigDict".to_string());
        let _ = writeln!(w, "    model_config = ConfigDict(populate_by_name=True)\n");
    };
}
