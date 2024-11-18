//! Post reconcile references after all types have been parsed.
//!
//! Types can be renamed via `serde(rename = "NewName")`. These types will get the new
//! name however we still need to see if we have any other types that reference the renamed type
//! and update those references accordingly.
use crate::{
    language::CrateName,
    parser::ParsedData,
    rust_types::{RustEnum, RustEnumVariant, RustType, SpecialRustType},
    visitors::ImportedType,
};
use log::{debug, info};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    mem,
};

/// A mapping of original type names to a mapping of crate name to new name.
type RenamedTypes = HashMap<String, HashMap<CrateName, String>>;

/// Update any type references that have the refenced type renamed via `serde(rename)`.
pub fn reconcile_aliases(crate_parsed_data: &mut BTreeMap<CrateName, ParsedData>) {
    let serde_renamed = collect_serde_renames(crate_parsed_data);

    for (crate_name, parsed_data) in crate_parsed_data {
        let import_types = mem::take(&mut parsed_data.import_types);

        // update references to renamed ids in product types.
        for s in &mut parsed_data.structs {
            debug!("struct: {}", s.id.original);
            for f in &mut s.fields {
                check_type(crate_name, &serde_renamed, &import_types, &mut f.ty);
            }
        }

        // update references to renamed ids in sum types.
        for e in &mut parsed_data.enums {
            debug!("enum: {}", e.shared().id.original);
            match e {
                RustEnum::Unit(shared) => check_variant(
                    crate_name,
                    &serde_renamed,
                    &import_types,
                    &mut shared.variants,
                ),
                RustEnum::Algebraic { shared, .. } => check_variant(
                    crate_name,
                    &serde_renamed,
                    &import_types,
                    &mut shared.variants,
                ),
            }
        }

        // update references to renamed ids in aliases.
        for a in &mut parsed_data.aliases {
            check_type(crate_name, &serde_renamed, &import_types, &mut a.r#type);
        }

        // Apply sorting to types for deterministic output.
        parsed_data.structs.sort();
        parsed_data.enums.sort();
        parsed_data.aliases.sort();

        // put back our import types for file generation.
        parsed_data.import_types = import_types;
    }
}

/// Traverse all the parsed typeshare data and collect all types that have been renamed
/// via `serde(rename)` into a mapping of original name to renamed name.
fn collect_serde_renames(crate_parsed_data: &BTreeMap<CrateName, ParsedData>) -> RenamedTypes {
    crate_parsed_data
        .iter()
        .flat_map(|(crate_name, parsed_data)| {
            parsed_data
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
                .chain(parsed_data.aliases.iter().flat_map(|e| {
                    e.id.serde_rename
                        .then(|| (e.id.original.to_string(), e.id.renamed.to_string()))
                }))
                .map(|(original, renamed)| (crate_name.to_owned(), (original, renamed)))
        })
        .fold(
            HashMap::new(),
            |mut mapping, (crate_name, (original, renamed))| {
                let name_map = mapping.entry(original).or_default();
                name_map.insert(crate_name.to_owned(), renamed);
                mapping
            },
        )
}

fn check_variant(
    crate_name: &CrateName,
    serde_renamed: &RenamedTypes,
    imported_types: &HashSet<ImportedType>,
    variants: &mut Vec<RustEnumVariant>,
) {
    for v in variants {
        match v {
            RustEnumVariant::Unit(_) => (),
            RustEnumVariant::Tuple { ty, .. } => {
                check_type(crate_name, serde_renamed, imported_types, ty);
            }
            RustEnumVariant::AnonymousStruct { fields, .. } => {
                for f in fields {
                    check_type(crate_name, serde_renamed, imported_types, &mut f.ty);
                }
            }
        }
    }
}

fn check_type(
    crate_name: &CrateName,
    serde_renamed: &RenamedTypes,
    import_types: &HashSet<ImportedType>,
    ty: &mut RustType,
) {
    debug!("checking type: {ty:?}");
    match ty {
        RustType::Generic { parameters, .. } => {
            for ty in parameters {
                check_type(crate_name, serde_renamed, import_types, ty);
            }
        }
        RustType::Special(s) => match s {
            SpecialRustType::Vec(ty) => {
                check_type(crate_name, serde_renamed, import_types, ty);
            }
            SpecialRustType::Array(ty, _) => {
                check_type(crate_name, serde_renamed, import_types, ty);
            }
            SpecialRustType::Slice(ty) => {
                check_type(crate_name, serde_renamed, import_types, ty);
            }
            SpecialRustType::HashMap(ty1, ty2) => {
                check_type(crate_name, serde_renamed, import_types, ty1);
                check_type(crate_name, serde_renamed, import_types, ty2);
            }
            SpecialRustType::Option(ty) => {
                check_type(crate_name, serde_renamed, import_types, ty);
            }
            _ => (),
        },
        RustType::Simple { id } => {
            debug!("{crate_name} looking up original name {id}");

            if let Some(renamed) = resolve_renamed(crate_name, serde_renamed, import_types, id) {
                info!("renaming type from {id} to {renamed}");
                *id = renamed.to_owned();
            }
        }
    }
}

fn resolve_renamed(
    crate_name: &CrateName,
    serde_renamed: &RenamedTypes,
    import_types: &HashSet<ImportedType>,
    id: &str,
) -> Option<String> {
    let name_map = serde_renamed.get(id)?;

    // Find in imports.
    import_types
        .iter()
        .filter(|i| &i.type_name == id)
        .find_map(|import_ref| name_map.get(&import_ref.base_crate))
        // Fallback to looking up in our current namespace.
        .or_else(|| name_map.get(crate_name))
        .map(ToOwned::to_owned)
}
