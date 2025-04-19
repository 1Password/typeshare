//! Visitors to collect various items from the AST.

use std::{collections::HashSet, iter, mem};

use syn::{visit::Visit, Attribute, ItemUse, UseTree};
use typeshare_model::{
    parsed_data::ImportedType,
    prelude::{CrateName, FilesMode, RustEnumVariant, RustType, TypeName},
};

use crate::{
    parser::{
        self, has_typeshare_annotation, parse_const, parse_enum, parse_struct, parse_type_alias,
        ParsedData, RustItem,
    },
    ParseError, ParseErrorSet,
};

/// List of some popular crate names that we can ignore
/// during import parsing.
const IGNORED_BASE_CRATES: &[&str] = &[
    "std",
    "serde",
    "serde_json",
    "typeshare",
    "once_cell",
    "itertools",
    "anyhow",
    "thiserror",
    "quote",
    "syn",
    "clap",
    "tokio",
    "reqwest",
    "regex",
    "http",
    "time",
    "axum",
    "either",
    "chrono",
    "base64",
    "rayon",
    "ring",
    "zip",
    "neon",
];

/// List of reference types or imported types we can ignore during import parsing.
const IGNORED_TYPES: &[&str] = &["Option", "String", "Vec", "HashMap", "T", "I54", "U53"];

/// An import visitor that collects all use or
/// qualified referenced items.
pub struct TypeShareVisitor<'a> {
    parsed_data: ParsedData,

    ignored_types: &'a [&'a str],
    mode: FilesMode<&'a CrateName>,
    errors: Vec<ParseError>,

    // If present, only things that would exist under any of OSes are generated
    // (as determined by `#[cfg(target_os=...)]`)
    target_os: Option<&'a [&'a str]>,
}

impl<'a> TypeShareVisitor<'a> {
    /// Create an import visitor for a given crate name.
    pub fn new(
        ignored_types: &'a [&'a str],
        mode: FilesMode<&'a CrateName>,
        target_os: Option<&'a [&'a str]>,
    ) -> Self {
        Self {
            parsed_data: ParsedData::default(),
            ignored_types,
            mode,
            errors: Vec::new(),
            target_os,
        }
    }

    /// Consume the visitor and return parsed data.
    pub fn parsed_data(mut self) -> Result<ParsedData, ParseErrorSet> {
        ParseErrorSet::collect(mem::take(&mut self.errors)).map(|()| self.parsed_data)
    }

    fn multi_file(&self) -> bool {
        self.mode.is_multi()
    }

    fn collect_result(&mut self, result: Result<RustItem, ParseError>) {
        match result {
            Ok(data) => self.parsed_data.add(data),
            Err(error) => self.errors.push(error),
        }
    }

    fn target_os_good(&self, attrs: &[Attribute]) -> bool {
        match self.target_os {
            Some(valid) => parser::check_target_os(attrs, valid),
            None => true,
        }
    }
}

impl<'ast, 'a> Visit<'ast> for TypeShareVisitor<'a> {
    /// Find any reference types that are not part of
    /// the `use` import statements.
    fn visit_path(&mut self, p: &'ast syn::Path) {
        // TODO: implement this as a part of import detection.
        // TODO: paths are used in a lot of places; make sure that we only
        // care about paths that are part of type definitions.

        // if !self.multi_file() {
        //     return;
        // }

        // let extract_root_and_types = |p: &syn::Path| {
        //     // TODO: the first part here may not be a crate name but a module name defined
        //     // in a use statement.
        //     //
        //     // Ex:
        //     // use some_crate::some_module;
        //     //
        //     // struct SomeType {
        //     //     field: some_module::RefType
        //     // }
        //     //
        //     // vist_path would be after vist_item_use so we could retain imported module references
        //     // and reconcile aftewards. visit_item_use would have to retain non type import types
        //     // which it discards right now.
        //     //
        //     let crate_candidate = CrateName::new(p.segments.first()?.ident.to_string());
        //     let type_candidate = TypeName::new(&p.segments.last()?.ident);

        //     (accept_crate(&crate_candidate)
        //         && accept_type(&type_candidate)
        //         && !self.ignored_types.contains(&type_candidate.as_str())
        //         && crate_candidate.as_str() != type_candidate.as_str())
        //     .then(|| {
        //         // resolve crate and super aliases into the crate name.
        //         let base_crate = if crate_candidate == "crate"
        //             || crate_candidate == "super"
        //             || crate_candidate == "self"
        //         {
        //             self.parsed_data.crate_name.clone()
        //         } else {
        //             crate_candidate
        //         };
        //         ImportedType {
        //             base_crate: CrateName::from(base_crate),
        //             type_name: type_candidate,
        //         }
        //     })
        // };

        // if let Some(imported_type) = extract_root_and_types(p) {
        //     self.parsed_data.import_types.insert(imported_type);
        // }
        // syn::visit::visit_path(self, p);
    }

    /// Collect referenced imports.
    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        // TODO: implement this as a part of import detection.
        // TODO: make sure that use items in submodules are skipped

        // if !self.multi_file() {
        //     return;
        // }

        // self.parsed_data.import_types.extend(
        //     parse_import(i, &self.parsed_data.crate_name)
        //         .filter(|imp| !self.ignored_types.contains(&imp.type_name.as_str())),
        // );

        // pub use other_crate::TypeshareType;
        // syn::visit::visit_item_use(self, i);
    }

    /// Collect rust structs.
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        if has_typeshare_annotation(&i.attrs) && self.target_os_good(&i.attrs) {
            self.collect_result(parse_struct(i, self.target_os));
        }

        syn::visit::visit_item_struct(self, i);
    }

    /// Collect rust enums.
    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        if has_typeshare_annotation(&i.attrs) && self.target_os_good(&i.attrs) {
            self.collect_result(parse_enum(i, self.target_os));
        }

        syn::visit::visit_item_enum(self, i);
    }

    /// Collect rust type aliases.
    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        if has_typeshare_annotation(&i.attrs) && self.target_os_good(&i.attrs) {
            self.collect_result(parse_type_alias(i));
        }

        syn::visit::visit_item_type(self, i);
    }

    fn visit_item_const(&mut self, i: &'ast syn::ItemConst) {
        if has_typeshare_annotation(&i.attrs) && self.target_os_good(&i.attrs) {
            self.collect_result(parse_const(i));
        }

        syn::visit::visit_item_const(self, i);
    }
}

/// Exclude popular crates that won't be typeshared.
fn accept_crate(crate_name: &CrateName) -> bool {
    IGNORED_BASE_CRATES
        .iter()
        .all(|&ignored| crate_name != ignored)
        && crate_name
            .as_str()
            .chars()
            .next()
            .map(|c| c.is_lowercase())
            .unwrap_or(false)
}

/// Accept types which start with an uppercase character.
pub(crate) fn accept_type(type_name: &TypeName) -> bool {
    IGNORED_TYPES.iter().all(|ignored| type_name != ignored)
        && type_name
            .as_str()
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
}

struct ItemUseIter<'a> {
    use_tree: Vec<&'a UseTree>,
    crate_name: &'a CrateName,
    base_name: Option<String>,
}

impl<'a> ItemUseIter<'a> {
    pub fn new(use_tree: &'a UseTree, crate_name: &'a CrateName) -> Self {
        Self {
            use_tree: vec![use_tree],
            crate_name,
            base_name: None,
        }
    }

    fn resolve_crate_name(&self) -> CrateName {
        let base_name = self.base_name();

        if base_name == "crate" || base_name == "super" || base_name == "self" {
            self.crate_name.clone()
        } else {
            CrateName::new(base_name)
        }
    }

    fn add_name(&mut self, ident: &syn::Ident) {
        if self.base_name.is_none() {
            self.base_name = Some(ident.to_string());
        }
    }

    fn base_name(&self) -> String {
        self.base_name
            .as_ref()
            .cloned()
            .expect("base name not in use statement?")
    }
}

impl<'a> Iterator for ItemUseIter<'a> {
    type Item = ImportedType;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(use_tree) = self.use_tree.pop() {
            match use_tree {
                syn::UseTree::Path(path) => {
                    self.add_name(&path.ident);
                    self.use_tree.push(&path.tree);
                }
                syn::UseTree::Name(name) => {
                    let type_name = TypeName::new(&name.ident);
                    let base_crate = self.resolve_crate_name();
                    if accept_crate(&base_crate) && accept_type(&type_name) {
                        return Some(ImportedType {
                            base_crate,
                            type_name,
                        });
                    }
                }
                syn::UseTree::Rename(_rename) => {
                    // TODO: I need to do something here.
                }
                syn::UseTree::Glob(_) => {
                    let base_crate = self.resolve_crate_name();
                    if accept_crate(&base_crate) {
                        return Some(ImportedType {
                            base_crate,
                            type_name: TypeName::new_static("*"),
                        });
                    }
                }
                syn::UseTree::Group(g) => {
                    self.use_tree.extend(g.items.iter());
                }
            }
        }

        None
    }
}

/// Yield all the type names including nested generic types. This only yields
/// "normal" type names, referring to user types or generics; it omits things
/// like integers and so on.
pub fn all_reference_type_names(ty: &RustType) -> impl Iterator<Item = &TypeName> + '_ {
    let mut type_stack = Vec::from([ty]);

    iter::from_fn(move || loop {
        break match type_stack.pop()? {
            // Special types don't have referencable names of their own, but
            // they do have parameters
            RustType::Special(ty) => {
                type_stack.extend(ty.parameters());
                continue;
            }
            RustType::Generic { id, parameters } => {
                type_stack.extend(parameters);
                Some(id)
            }
            RustType::Simple { id } => Some(id),
        };
    })
    .filter(|s| accept_type(s))
}

fn parse_import<'a>(
    item_use: &'a ItemUse,
    crate_name: &'a CrateName,
) -> impl Iterator<Item = ImportedType> + 'a {
    ItemUseIter::new(&item_use.tree, crate_name)
}

#[cfg(test)]
mod test {
    use super::{parse_import, TypeShareVisitor};
    use crate::visitors::ImportedType;
    use cool_asserts::assert_matches;
    use itertools::Itertools;
    use syn::{visit::Visit, File};
    use typeshare_model::prelude::{CrateName, FilesMode};

    #[test]
    fn test_parse_import_complex() {
        let rust_file = "
           use combined::{
                one::TypeOne,
                two::TypeThree,
                three::{TypeFour, TypeFive, four::TypeSix}
           };
           ";
        let file = syn::parse_str::<syn::File>(rust_file).unwrap();

        let parsed_imports = file
            .items
            .iter()
            .flat_map(|item| {
                if let syn::Item::Use(use_item) = item {
                    parse_import(use_item, &CrateName::new("my_crate".to_owned())).collect()
                } else {
                    Vec::new()
                }
            })
            .collect::<Vec<_>>();

        assert_matches!(parsed_imports,
            [
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "combined");
                    assert_eq!(type_name, "TypeSix");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "combined");
                    assert_eq!(type_name, "TypeFive");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "combined");
                    assert_eq!(type_name, "TypeFour");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "combined");
                    assert_eq!(type_name, "TypeThree");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "combined");
                    assert_eq!(type_name, "TypeOne");
                },
            ]
        );
    }

    #[test]
    fn test_parse_import() {
        let rust_file = "
            use std::sync::Arc;
            use quote::ToTokens;
            use std::collections::BTreeSet;
            use std::str::FromStr;
            use std::{collections::HashMap, convert::TryFrom};
            use some_crate::blah::*;
            use crate::types::{MyType, MyEnum};
            use super::some_module::{Hello, another_module::AnotherType, MyEnum};

        ";
        let file = syn::parse_str::<syn::File>(rust_file).unwrap();

        let parsed_imports = file
            .items
            .iter()
            .flat_map(|item| {
                if let syn::Item::Use(use_item) = item {
                    parse_import(use_item, &CrateName::new("my_crate".to_owned())).collect()
                } else {
                    Vec::new()
                }
            })
            .rev()
            .collect::<Vec<_>>();

        assert_matches!(
            parsed_imports,
            [
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "my_crate");
                    assert_eq!(type_name, "Hello");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "my_crate");
                    assert_eq!(type_name, "AnotherType");
                },
                ImportedType {
                    base_crate,
                    type_name,
                }  => {
                    assert_eq!(base_crate, "my_crate");
                    assert_eq!(type_name, "MyEnum");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "my_crate");
                    assert_eq!(type_name, "MyType");
                },
                ImportedType {
                    base_crate,
                    type_name,
                }  => {
                    assert_eq!(base_crate, "my_crate");
                    assert_eq!(type_name, "MyEnum");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "some_crate");
                    assert_eq!(type_name, "*");
                },

            ]
        );
    }

    // #[test]
    // fn test_path_visitor() {
    //     let rust_code = "
    //         use std::sync::Arc;
    //         use quote::ToTokens;
    //         use std::collections::BTreeSet;
    //         use std::str::FromStr;
    //         use std::{collections::HashMap, convert::TryFrom};
    //         use some_crate::blah::*;
    //         use crate::types::{MyType, MyEnum};
    //         use super::some_module::{another_module::AnotherType, AnotherEnum};

    //         enum TestEnum {
    //             Variant,
    //             Another {
    //                 field: Option<some_crate::module::Type>
    //             }
    //         }

    //         struct S {
    //             f: crate::another::TypeName
    //         }
    //         ";

    //     let file: File = syn::parse_str(rust_code).unwrap();
    //     let crate_name = CrateName::new("my_crate".to_owned());
    //     let mut visitor = TypeShareVisitor::new(&[], FilesMode::Multi(&crate_name));
    //     visitor.visit_file(&file);

    //     let mut sorted_imports = visitor.parsed_data.import_types.into_iter().collect_vec();
    //     sorted_imports.sort_unstable_by(|lhs, rhs| {
    //         Ord::cmp(lhs.base_crate.as_str(), rhs.base_crate.as_str())
    //             .then_with(|| Ord::cmp(lhs.type_name.as_str(), rhs.type_name.as_str()))
    //     });

    //     assert_matches!(
    //         sorted_imports,
    //         [
    //             ImportedType {
    //                 base_crate,
    //                 type_name,
    //             }  => {
    //                 assert_eq!(base_crate, "my_crate");
    //                 assert_eq!(type_name, "AnotherEnum");
    //             },
    //             ImportedType {
    //                 base_crate,
    //                 type_name,
    //             } => {
    //                 assert_eq!(base_crate, "my_crate");
    //                 assert_eq!(type_name, "AnotherType");
    //             },
    //             ImportedType {
    //                 base_crate,
    //                 type_name,
    //             }  => {
    //                 assert_eq!(base_crate, "my_crate");
    //                 assert_eq!(type_name, "MyEnum");
    //             },
    //             ImportedType {
    //                 base_crate,
    //                 type_name,
    //             } => {
    //                 assert_eq!(base_crate, "my_crate");
    //                 assert_eq!(type_name, "MyType");
    //             },
    //             ImportedType {
    //                 base_crate,
    //                 type_name,
    //             }  => {
    //                 assert_eq!(base_crate, "my_crate");
    //                 assert_eq!(type_name, "TypeName");
    //             },
    //             ImportedType {
    //                 base_crate,
    //                 type_name,
    //             } => {
    //                 assert_eq!(base_crate, "some_crate");
    //                 assert_eq!(type_name, "*");
    //             },
    //             ImportedType {
    //                 base_crate,
    //                 type_name,
    //             }  => {
    //                 assert_eq!(base_crate, "some_crate");
    //                 assert_eq!(type_name, "Type");
    //             },
    //         ]
    //     );
    // }
}
