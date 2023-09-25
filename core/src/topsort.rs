use std::collections::{HashMap, HashSet};

use crate::parsed_types::{
    AlgebraicEnum, EnumVariant, Item, ParsedEnum, ParsedStruct, ParsedTypeAlias, SpecialType,
    TupleVariant, Type,
};

fn get_dependencies_from_type(
    tp: &Type,
    types: &HashMap<String, &Item>,
    res: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    match tp {
        Type::Generic { id, parameters } => {
            if let Some(&tp) = types.get(id) {
                if seen.insert(id.clone()) {
                    res.push(id.clone());
                    get_dependencies(tp, types, res, seen);
                    for parameter in parameters {
                        let id = parameter.id().to_string();
                        if let Some(&tp) = types.get(&id) {
                            if seen.insert(id.clone()) {
                                res.push(id.clone());
                                get_dependencies(tp, types, res, seen);
                                seen.remove(&id.clone());
                            }
                        }
                    }
                    seen.remove(&id.clone());
                }
            }
        }
        Type::Simple { id } => {
            if let Some(&tp) = types.get(id) {
                if seen.insert(id.clone()) {
                    res.push(id.clone());
                    get_dependencies(tp, types, res, seen);
                    seen.remove(&id.clone());
                }
            }
        }
        Type::Special(special) => match special {
            SpecialType::Map(kt, vt) => {
                get_dependencies_from_type(kt, types, res, seen);
                get_dependencies_from_type(vt, types, res, seen);
            }
            SpecialType::Option(inner) => {
                get_dependencies_from_type(inner, types, res, seen);
            }
            SpecialType::Vec(inner) => {
                get_dependencies_from_type(inner, types, res, seen);
            }
            _ => {}
        },
    };
    seen.remove(&tp.id().to_string());
}

fn get_enum_dependencies(
    enm: &ParsedEnum,
    types: &HashMap<String, &Item>,
    res: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    match enm {
        ParsedEnum::Unit(_) => {}
        ParsedEnum::Algebraic(AlgebraicEnum {
            tag_key: _,
            content_key: _,
            shared,
        }) => {
            if seen.insert(shared.id.original.to_string()) {
                res.push(shared.id.original.to_string());
                for variant in &shared.variants {
                    match variant {
                        EnumVariant::Unit(_) => {}
                        EnumVariant::AnonymousStruct(_) => {}
                        EnumVariant::Tuple(TupleVariant { ty, .. }) => {
                            get_dependencies_from_type(ty, types, res, seen)
                        }
                    }
                }
                seen.remove(&shared.id.original.to_string());
            }
        }
        ParsedEnum::SerializedAs { .. } => {
            todo!("TopSort Serialize As enums")
        }
    }
}

fn get_struct_dependencies(
    strct: &ParsedStruct,
    types: &HashMap<String, &Item>,
    res: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    match strct {
        ParsedStruct::TraditionalStruct { fields, shared: _ } => {
            if seen.insert(strct.id.original.to_string()) {
                for field in fields {
                    get_dependencies_from_type(&field.ty, types, res, seen)
                }
                seen.remove(&strct.id.original.to_string());
            }
        }
        ParsedStruct::SerializedAs { .. } => {
            todo!("TopSort Serialize As structs")
        }
    }
}

fn get_type_alias_dependencies(
    ta: &ParsedTypeAlias,
    types: &HashMap<String, &Item>,
    res: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    if seen.insert(ta.id.original.to_string()) {
        get_dependencies_from_type(&ta.value_type, types, res, seen);
        for generic in &ta.generic_types.generics {
            if let Some(&thing) = types.get(generic) {
                get_dependencies(thing, types, res, seen)
            }
        }
        seen.remove(&ta.id.original.to_string());
    }
}

fn get_dependencies(
    thing: &Item,
    types: &HashMap<String, &Item>,
    res: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    match thing {
        Item::Enum(en) => get_enum_dependencies(en, types, res, seen),
        Item::Struct(strct) => get_struct_dependencies(strct, types, res, seen),
        Item::Alias(alias) => get_type_alias_dependencies(alias, types, res, seen),
    }
}

fn get_index(thing: &Item, things: &[&Item]) -> usize {
    things
        .iter()
        .position(|&r| r == thing)
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

pub fn topsort(things: Vec<&Item>) -> Vec<&Item> {
    let types = HashMap::from_iter(things.iter().map(|&thing| {
        let id = match thing {
            Item::Enum(e) => match e {
                ParsedEnum::Algebraic(AlgebraicEnum { shared, .. }) => shared.id.original.clone(),
                ParsedEnum::Unit(shared) => shared.id.original.clone(),
                ParsedEnum::SerializedAs { .. } => {
                    todo!("TopSort Serialize As enums")
                }
            },
            Item::Struct(strct) => strct.id.original.clone(),
            Item::Alias(ta) => ta.id.original.clone(),
        };
        (id, thing)
    }));
    let dag: Vec<Vec<usize>> = things
        .iter()
        .map(|&thing| {
            let mut deps = vec![];
            get_dependencies(thing, &types, &mut deps, &mut HashSet::new());
            deps.iter()
                .map(|dep| get_index(types.get(dep).unwrap(), &things))
                .collect()
        })
        .collect();
    let sorted = toposort_impl(&dag);
    sorted.iter().map(|&idx| things[idx]).collect()
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
