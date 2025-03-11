use std::collections::{HashMap, HashSet};

use typeshare_model::prelude::*;

use crate::writer::BorrowedRustItem;

fn get_dependencies_from_type<'a>(
    tp: &'a RustType,
    types: &'a HashMap<TypeName, BorrowedRustItem<'a>>,
    res: &mut Vec<&'a TypeName>,
    seen: &mut HashSet<&'a TypeName>,
) {
    match tp {
        RustType::Generic { id, parameters } => {
            if let Some(&tp) = types.get(id) {
                if seen.insert(&id) {
                    res.push(&id);
                    get_dependencies(tp, types, res, seen);
                    for parameter in parameters {
                        let id = parameter.id();
                        if let Some(&tp) = types.get(id) {
                            if seen.insert(&id) {
                                res.push(&id);
                                get_dependencies(tp, types, res, seen);
                                seen.remove(&id.clone());
                            }
                        }
                    }
                    seen.remove(&id.clone());
                }
            }
        }
        RustType::Simple { id } => {
            if let Some(tp) = types.get(id) {
                if seen.insert(id) {
                    res.push(&id);
                    get_dependencies(*tp, types, res, seen);
                    seen.remove(&id.clone());
                }
            }
        }
        RustType::Special(special) => match special {
            SpecialRustType::HashMap(kt, vt) => {
                get_dependencies_from_type(kt, types, res, seen);
                get_dependencies_from_type(vt, types, res, seen);
            }
            SpecialRustType::Option(inner) => {
                get_dependencies_from_type(inner, types, res, seen);
            }
            SpecialRustType::Vec(inner) => {
                get_dependencies_from_type(inner, types, res, seen);
            }
            _ => {}
        },
    };

    seen.remove(tp.id());
}

fn get_enum_dependencies<'a>(
    enm: &'a RustEnum,
    types: &'a HashMap<TypeName, BorrowedRustItem<'a>>,
    res: &mut Vec<&'a TypeName>,
    seen: &mut HashSet<&'a TypeName>,
) {
    match enm {
        RustEnum::Unit(_) => {}
        RustEnum::Algebraic {
            tag_key: _,
            content_key: _,
            shared,
        } => {
            if seen.insert(&shared.id.original) {
                res.push(&shared.id.original);
                for variant in &shared.variants {
                    match variant {
                        RustEnumVariant::Unit(_) => {}
                        RustEnumVariant::AnonymousStruct {
                            fields: _,
                            shared: _,
                        } => {}
                        RustEnumVariant::Tuple { ty, shared: _ } => {
                            get_dependencies_from_type(ty, types, res, seen)
                        }
                    }
                }
                seen.remove(&shared.id.original);
            }
        }
    }
}

fn get_struct_dependencies<'a>(
    strct: &'a RustStruct,
    types: &'a HashMap<TypeName, BorrowedRustItem<'a>>,
    res: &mut Vec<&'a TypeName>,
    seen: &mut HashSet<&'a TypeName>,
) {
    if seen.insert(&strct.id.original) {
        for field in &strct.fields {
            get_dependencies_from_type(&field.ty, types, res, seen)
        }
        seen.remove(&strct.id.original);
    }
}

fn get_type_alias_dependencies<'a>(
    ta: &'a RustTypeAlias,
    types: &'a HashMap<TypeName, BorrowedRustItem<'a>>,
    res: &mut Vec<&'a TypeName>,
    seen: &mut HashSet<&'a TypeName>,
) {
    if seen.insert(&ta.id.original) {
        get_dependencies_from_type(&ta.r#type, types, res, seen);
        for generic in &ta.generic_types {
            if let Some(thing) = types.get(generic) {
                get_dependencies(*thing, types, res, seen)
            }
        }
        seen.remove(&ta.id.original);
    }
}

// fn get_const_dependencies(
//     c: &RustConst,
//     types: &HashMap<TypeName, &RustItem>,
//     res: &mut Vec<String>,
//     seen: &mut HashSet<String>,
// ) {
//     if seen.insert(c.id.original.to_string()) {
//         get_dependencies_from_type(&c.r#type, types, res, seen);
//         seen.remove(&c.id.original.to_string());
//     }
// }

fn get_dependencies<'a>(
    thing: BorrowedRustItem<'a>,
    types: &'a HashMap<TypeName, BorrowedRustItem<'a>>,
    res: &mut Vec<&'a TypeName>,
    seen: &mut HashSet<&'a TypeName>,
) {
    match thing {
        BorrowedRustItem::Enum(en) => get_enum_dependencies(en, types, res, seen),
        BorrowedRustItem::Struct(strct) => get_struct_dependencies(strct, types, res, seen),
        BorrowedRustItem::Alias(alias) => get_type_alias_dependencies(alias, types, res, seen),
        // RustItem::Const(c) => get_const_dependencies(c, types, res, seen),
    }
}

fn get_index(thing: BorrowedRustItem<'_>, things: &[BorrowedRustItem<'_>]) -> usize {
    things
        .iter()
        .position(|r| *r == thing)
        .expect("Unable to find thing in things!")
}

#[allow(clippy::ptr_arg)] // Ignored due to false positive
fn toposort_impl(graph: &Vec<Vec<usize>>) -> Vec<usize> {
    fn inner(
        graph: &Vec<Vec<usize>>,
        nodes: &Vec<usize>,
        res: &mut Vec<usize>,
        processed: &mut Vec<usize>,
        seen: &mut Vec<usize>,
    ) {
        for dependant in nodes {
            if !processed.contains(dependant) {
                if !seen.contains(dependant) {
                    seen.push(*dependant);
                } else {
                    // cycle
                    return;
                }
                // recurse
                let dependencies = &graph[*dependant];
                inner(graph, dependencies, res, processed, seen);
                if let Some(position) = seen.iter().position(|&other| other == *dependant) {
                    seen.remove(position);
                }
                processed.push(*dependant);
                res.push(*dependant);
            }
        }
    }
    let mut res = vec![];
    let mut seen = vec![];
    let mut processed = vec![];
    inner(
        graph,
        &(0..graph.len()).collect(),
        &mut res,
        &mut processed,
        &mut seen,
    );
    res
}

pub(crate) fn topsort(things: &mut [BorrowedRustItem<'_>]) {
    let types = HashMap::from_iter(things.iter().map(|thing| {
        let id = match thing {
            BorrowedRustItem::Enum(e) => match e {
                RustEnum::Algebraic {
                    tag_key: _,
                    content_key: _,
                    shared,
                } => shared.id.original.clone(),
                RustEnum::Unit(shared) => shared.id.original.clone(),
            },
            BorrowedRustItem::Struct(strct) => strct.id.original.clone(),
            BorrowedRustItem::Alias(ta) => ta.id.original.clone(),
            //   RustItem::Const(c) => c.id.original.clone(),
        };
        (id, *thing)
    }));

    let dag: Vec<Vec<usize>> = things
        .iter()
        .map(|&thing| {
            let mut deps = Vec::new();
            get_dependencies(thing, &types, &mut deps, &mut HashSet::new());
            deps.iter()
                .map(|&dep| get_index(*types.get(dep).unwrap(), things))
                .collect()
        })
        .collect();

    sort_by_indices(things, toposort_impl(&dag));
}

/// In place sort of array using provided indices.
pub(crate) fn sort_by_indices<T>(data: &mut [T], mut indices: Vec<usize>) {
    for idx in 0..data.len() {
        if indices[idx] != idx {
            let mut current_idx = idx;
            loop {
                let target_idx = indices[current_idx];
                indices[current_idx] = current_idx;
                if indices[target_idx] == target_idx {
                    break;
                }
                data.swap(current_idx, target_idx);
                current_idx = target_idx;
            }
        }
    }
}

#[test]
fn test_toposort_impl() {
    let dag = vec![vec![], vec![0], vec![0, 1]];
    let res = toposort_impl(&dag);
    assert_eq!(res, vec![0, 1, 2])
}

#[test]
fn test_toposort_impl_cycles() {
    let dag = vec![vec![1], vec![0], vec![1]];
    let res = toposort_impl(&dag);
    assert!((res == vec![0, 1, 2]) || (res == vec![1, 0, 2]))
}
