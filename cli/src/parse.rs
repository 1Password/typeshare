//! Source file parsing.
use anyhow::Context;
use ignore::WalkBuilder;
use log::{debug, info};
use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap},
    path::PathBuf,
};
use typeshare_core::{
    context::{ParseContext, ParseFileContext},
    language::{CrateName, CrateTypes, SupportedLanguage, SINGLE_FILE_CRATE_NAME},
    parser::ParsedData,
    rust_types::{RustEnum, RustEnumVariant, RustType, SpecialRustType},
    RenameExt,
};

/// Input data for parsing each source file.
pub struct ParserInput {
    /// Rust source file path.
    file_path: PathBuf,
    /// File name source from crate for output.
    file_name: String,
    /// The crate name the source file belongs to.
    crate_name: CrateName,
}

/// Walk the source folder and collect all parser inputs.
pub fn parser_inputs(
    walker_builder: WalkBuilder,
    language_type: SupportedLanguage,
    multi_file: bool,
) -> impl Iterator<Item = ParserInput> {
    walker_builder
        .build()
        .filter_map(Result::ok)
        .filter(|dir_entry| !dir_entry.path().is_dir())
        .filter_map(move |dir_entry| {
            let crate_name = if multi_file {
                CrateName::find_crate_name(dir_entry.path())?
            } else {
                SINGLE_FILE_CRATE_NAME
            };
            let file_path = dir_entry.path().to_path_buf();
            let file_name = output_file_name(language_type, &crate_name);
            Some(ParserInput {
                file_path,
                file_name,
                crate_name,
            })
        })
}

/// The output file name to write to.
fn output_file_name(language_type: SupportedLanguage, crate_name: &CrateName) -> String {
    let extension = language_type.language_extension();

    let snake_case = || format!("{crate_name}.{extension}");
    let pascal_case = || format!("{}.{extension}", crate_name.to_string().to_pascal_case());

    match language_type {
        SupportedLanguage::Go => snake_case(),
        SupportedLanguage::Kotlin => snake_case(),
        SupportedLanguage::Scala => snake_case(),
        SupportedLanguage::Swift => pascal_case(),
        SupportedLanguage::TypeScript => snake_case(),
    }
}

/// Collect all the typeshared types into a mapping of crate names to typeshared types. This
/// mapping is used to lookup and generated import statements for generated files.
pub fn all_types(file_mappings: &BTreeMap<CrateName, ParsedData>) -> CrateTypes {
    file_mappings
        .iter()
        .map(|(crate_name, parsed_data)| (crate_name, parsed_data.type_names.clone()))
        .fold(
            HashMap::new(),
            |mut import_map: CrateTypes, (crate_name, type_names)| {
                match import_map.entry(crate_name.clone()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().extend(type_names);
                    }
                    Entry::Vacant(e) => {
                        e.insert(type_names);
                    }
                }
                import_map
            },
        )
}

/// Collect all the parsed sources into a mapping of crate name to parsed data.
pub fn parse_input(
    mut inputs: impl Iterator<Item = ParserInput>,
    parse_context: &ParseContext,
) -> anyhow::Result<BTreeMap<CrateName, ParsedData>> {
    inputs.try_fold(
        BTreeMap::new(),
        |mut parsed_crates: BTreeMap<CrateName, ParsedData>,
         ParserInput {
             file_path,
             file_name,
             crate_name,
         }| {
            let fp = file_path.as_os_str().to_str().unwrap_or("").to_string();

            let parse_file_context = ParseFileContext {
                source_code: std::fs::read_to_string(&file_path)
                    .with_context(|| format!("Failed to read input: {file_path:?}"))?,
                crate_name: crate_name.clone(),
                file_name: file_name.clone(),
                file_path,
            };

            let parsed_result = typeshare_core::parser::parse(parse_context, parse_file_context)
                .with_context(|| format!("Failed to parse: {fp}"))?;

            if let Some(parsed_data) = parsed_result {
                parsed_crates
                    .entry(crate_name)
                    .or_default()
                    .add(parsed_data);
            }

            Ok(parsed_crates)
        },
    )
}

/// Update any type references that have the refenced type renamed via `serde(rename)`.
pub fn reconcile_aliases(crate_parsed_data: &mut BTreeMap<CrateName, ParsedData>) {
    for (_crate_name, parsed_data) in crate_parsed_data {
        let serde_renamed = parsed_data
            .structs
            .iter()
            .flat_map(|s| {
                s.id.serde_rename
                    .then_some((s.id.original.to_string(), s.id.renamed.to_string()))
            })
            .chain(parsed_data.enums.iter().flat_map(|e| {
                e.shared().id.serde_rename.then_some((
                    e.shared().id.original.to_string(),
                    e.shared().id.renamed.to_string(),
                ))
            }))
            .collect::<HashMap<_, _>>();

        // find references to original ids and update accordingly
        for s in &mut parsed_data.structs {
            debug!("struct: {}", s.id.original);
            for f in &mut s.fields {
                check_type(&serde_renamed, &mut f.ty);
            }
        }

        for e in &mut parsed_data.enums {
            debug!("enum: {}", e.shared().id.original);
            match e {
                RustEnum::Unit(shared) => check_variant(&serde_renamed, &mut shared.variants),
                RustEnum::Algebraic { shared, .. } => {
                    check_variant(&serde_renamed, &mut shared.variants)
                }
            }
        }
    }
}

fn check_variant(serde_renamed: &HashMap<String, String>, variants: &mut Vec<RustEnumVariant>) {
    for v in variants {
        match v {
            RustEnumVariant::Unit(_) => (),
            RustEnumVariant::Tuple { ty, .. } => {
                check_type(serde_renamed, ty);
            }
            RustEnumVariant::AnonymousStruct { fields, .. } => {
                for f in fields {
                    check_type(serde_renamed, &mut f.ty);
                }
            }
        }
    }
}

fn check_type(serde_renamed: &HashMap<String, String>, ty: &mut RustType) {
    debug!("checking type: {ty:?}");
    match ty {
        RustType::Generic { parameters, .. } => {
            for ty in parameters {
                check_type(serde_renamed, ty);
            }
        }
        RustType::Special(s) => match s {
            SpecialRustType::Vec(ty) => {
                check_type(serde_renamed, ty);
            }
            SpecialRustType::Array(ty, _) => {
                check_type(serde_renamed, ty);
            }
            SpecialRustType::Slice(ty) => {
                check_type(serde_renamed, ty);
            }
            SpecialRustType::HashMap(ty1, ty2) => {
                check_type(serde_renamed, ty1);
                check_type(serde_renamed, ty2);
            }
            SpecialRustType::Option(ty) => {
                check_type(serde_renamed, ty);
            }
            _ => (),
        },
        RustType::Simple { id } => {
            if let Some(alias) = serde_renamed.get(id) {
                info!("renaming type from {id} to {alias}");
                *id = alias.to_string()
            }
        }
    }
}
