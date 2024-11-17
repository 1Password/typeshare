//! Post reconcile references after all types have been parsed.
//!
//! Types can be renamed via `serde(rename = "NewName")`. These types will get the new
//! name however we still need to see if we have any other types that reference the renamed type
//! and update those references accordingly.
use crate::{
    language::CrateName,
    parser::ParsedData,
    rust_types::{RustEnum, RustEnumVariant, RustType, SpecialRustType},
};
use log::{debug, info};
use std::collections::{BTreeMap, HashMap};

/// Update any type references that have the refenced type renamed via `serde(rename)`.
pub fn reconcile_aliases(crate_parsed_data: &mut BTreeMap<CrateName, ParsedData>) {
    for (_crate_name, parsed_data) in crate_parsed_data {
        // TODO: This assumes the reference and reference type are both in the same crate namespace.
        // This will be fine if multi-file is not used however if multi-file is used and the reference
        // type and it's references are in separate crates it will not.
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
            .chain(parsed_data.aliases.iter().flat_map(|e| {
                e.id.serde_rename
                    .then(|| (e.id.original.to_string(), e.id.renamed.to_string()))
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

        for a in &mut parsed_data.aliases {
            check_type(&serde_renamed, &mut a.r#type);
        }

        parsed_data.structs.sort();
        parsed_data.enums.sort();
        parsed_data.aliases.sort();
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
