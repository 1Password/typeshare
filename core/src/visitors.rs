//! Visitors to collect various items from the AST.
use std::{collections::HashSet, path::PathBuf};

use crate::{
    parser::{
        has_typeshare_annotation, parse_enum, parse_struct, parse_type_alias, ErrorInfo,
        ParseError, ParsedData,
    },
    rust_types::{RustEnumVariant, RustItem},
};
use syn::{visit::Visit, ItemUse, UseTree};

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
#[derive(Default)]
pub struct TypeShareVisitor {
    parsed_data: ParsedData,
    file_path: PathBuf,
}

impl TypeShareVisitor {
    /// Create an import visitor for a given crate name.
    pub fn new(crate_name: String, file_name: String, file_path: PathBuf) -> Self {
        Self {
            parsed_data: ParsedData::new(crate_name, file_name),
            file_path,
        }
    }

    /// Consume the visitor and return parsed data.
    pub fn parsed_data(self) -> ParsedData {
        let mut s = self;
        s.reconcile_referenced_types();
        s.parsed_data
    }

    fn collect_result(&mut self, result: Result<RustItem, ParseError>) {
        match result {
            Ok(data) => self.parsed_data.push(data),
            Err(error) => self.parsed_data.errors.push(ErrorInfo {
                crate_name: self.parsed_data.crate_name.clone(),
                file_name: self.parsed_data.file_name.clone(),
                error,
            }),
        }
    }

    /// After collecting all imports we now want to retain only those
    /// that are referenced by the typeshared types.
    fn reconcile_referenced_types(&mut self) {
        // Build up a set of all the types that are referenced by
        // the typshared types we have parsed.
        let mut all_references = HashSet::new();

        all_references.extend(
            self.parsed_data
                .structs
                .iter()
                .flat_map(|s| s.fields.iter())
                .flat_map(|f| f.ty.all_reference_type_names()),
        );

        for v in self
            .parsed_data
            .enums
            .iter()
            .flat_map(|e| e.shared().variants.iter())
        {
            match v {
                RustEnumVariant::Unit(_) => (),
                RustEnumVariant::Tuple { ty, .. } => {
                    all_references.extend(ty.all_reference_type_names());
                }
                RustEnumVariant::AnonymousStruct { fields, .. } => {
                    all_references
                        .extend(fields.iter().flat_map(|f| f.ty.all_reference_type_names()));
                }
            }
        }

        all_references.extend(
            self.parsed_data
                .type_names
                .iter()
                .filter(|s| accept_type(s))
                .map(|s| s.as_str()),
        );

        // Build a set of a all type names.
        let local_types = self
            .parsed_data
            .type_names
            .iter()
            .map(|s| s.as_str())
            .collect::<HashSet<_>>();

        // Lookup a type name against parsed imports.
        let find_type = |name: &str| {
            let found = self
                .parsed_data
                .import_types
                .iter()
                .find(|imp| imp.type_name == name)
                .into_iter()
                .next()
                .cloned();

            if found.is_none() {
                println!(
                    "Failed to lookup \"{name}\" in crate \"{}\" for file \"{}\"",
                    self.parsed_data.crate_name,
                    self.file_path.as_os_str().to_str().unwrap_or_default()
                );
            }

            found
        };

        // Lookup all the references that are not defined locally. Subtract
        // all local types defined in the module.
        let mut diff = all_references
            .difference(&local_types)
            .copied()
            .flat_map(find_type)
            .collect::<HashSet<_>>();

        // Move back the wildcard import types.
        diff.extend(
            self.parsed_data
                .import_types
                .drain()
                .filter(|imp| imp.type_name == "*"),
        );

        self.parsed_data.import_types = diff;
    }
}

impl<'ast> Visit<'ast> for TypeShareVisitor {
    /// Find any reference types that are not part of
    /// the `use` import statements.
    fn visit_path(&mut self, p: &'ast syn::Path) {
        fn extract_root_and_types(p: &syn::Path, crate_name: &str) -> Option<ImportedType> {
            // TODO: the first part here may not be a crate name but a module name defined
            // in a use statement.
            //
            // Ex:
            // use some_crate::some_module;
            //
            // struct SomeType {
            //     field: some_module::RefType
            // }
            //
            // vist_path would be after vist_item_use so we could retain imported module references
            // and reconcile aftewards. visit_item_use would have to retain not type import types
            // which it discards right now.
            //
            let first = p.segments.first()?.ident.to_string();
            let last = p.segments.last()?.ident.to_string();

            accept_crate(&first)
                .then_some(())
                .and_then(|_| accept_type(&last).then_some(()))?;

            (first != last).then(|| {
                // resolve crate and super aliases into the crate name.
                let base_crate = if first == "crate" || first == "super" || first == "self" {
                    crate_name.to_string()
                } else {
                    first
                };
                ImportedType {
                    base_crate,
                    type_name: last,
                }
            })
        }

        if let Some(imported_type) = extract_root_and_types(p, &self.parsed_data.crate_name) {
            self.parsed_data.import_types.insert(imported_type);
        }
        syn::visit::visit_path(self, p);
    }

    /// Collect referenced imports.
    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        self.parsed_data
            .import_types
            .extend(parse_import(i, &self.parsed_data.crate_name));
        syn::visit::visit_item_use(self, i);
    }

    /// Collect rust structs.
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        if has_typeshare_annotation(&i.attrs) {
            self.collect_result(parse_struct(i));
        }

        syn::visit::visit_item_struct(self, i);
    }

    /// Collect rust enums.
    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        if has_typeshare_annotation(&i.attrs) {
            self.collect_result(parse_enum(i));
        }

        syn::visit::visit_item_enum(self, i);
    }

    /// Collect rust type aliases.
    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        if has_typeshare_annotation(&i.attrs) {
            self.collect_result(parse_type_alias(i));
        }

        syn::visit::visit_item_type(self, i);
    }
}

/// Exclude popular crates that won't be typeshared.
fn accept_crate(crate_name: &str) -> bool {
    !IGNORED_BASE_CRATES.contains(&crate_name)
        && crate_name
            .chars()
            .next()
            .map(|c| c.is_lowercase())
            .unwrap_or(false)
}

/// Accept types which start with an uppercase character.
pub(crate) fn accept_type(type_name: &str) -> bool {
    type_name
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
        && !IGNORED_TYPES.contains(&type_name)
}

/// An imported type reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Ord, PartialOrd))]
pub struct ImportedType {
    /// Crate this type belongs to.
    pub base_crate: String,
    /// Type name.
    pub type_name: String,
}

struct ItemUseIter<'a> {
    use_tree: Vec<&'a UseTree>,
    crate_name: &'a str,
    base_name: Option<String>,
}

impl<'a> ItemUseIter<'a> {
    pub fn new(use_tree: &'a UseTree, crate_name: &'a str) -> Self {
        Self {
            use_tree: vec![use_tree],
            crate_name,
            base_name: None,
        }
    }

    fn resolve_crate_name(&self) -> String {
        let crate_name = self.base_name();
        if crate_name == "crate" || crate_name == "super" || crate_name == "self" {
            self.crate_name.to_string()
        } else {
            crate_name.to_string()
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
                    let type_name = name.ident.to_string();
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
                            type_name: "*".into(),
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

fn parse_import<'a>(
    item_use: &'a ItemUse,
    crate_name: &'a str,
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
                    parse_import(use_item, "my_crate").collect()
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
                    parse_import(use_item, "my_crate").collect()
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

    #[test]
    fn test_path_visitor() {
        let rust_code = "
            use std::sync::Arc;
            use quote::ToTokens;
            use std::collections::BTreeSet;
            use std::str::FromStr;
            use std::{collections::HashMap, convert::TryFrom};
            use some_crate::blah::*;
            use crate::types::{MyType, MyEnum};
            use super::some_module::{another_module::AnotherType, AnotherEnum};

            enum TestEnum {
                Variant,
                Another {
                    field: Option<some_crate::module::Type>
                }
            }

            struct S {
                f: crate::another::TypeName
            }
            ";

        let file: File = syn::parse_str(rust_code).unwrap();
        let mut visitor =
            TypeShareVisitor::new("my_crate".into(), "my_file".into(), "file_path".into());
        visitor.visit_file(&file);

        let mut sorted_imports = visitor.parsed_data.import_types.into_iter().collect_vec();
        sorted_imports.sort();

        assert_matches!(
            sorted_imports,
            [
                ImportedType {
                    base_crate,
                    type_name,
                }  => {
                    assert_eq!(base_crate, "my_crate");
                    assert_eq!(type_name, "AnotherEnum");
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
                    assert_eq!(type_name, "TypeName");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "some_crate");
                    assert_eq!(type_name, "*");
                },
                ImportedType {
                    base_crate,
                    type_name,
                }  => {
                    assert_eq!(base_crate, "some_crate");
                    assert_eq!(type_name, "Type");
                },
            ]
        );
    }
}
