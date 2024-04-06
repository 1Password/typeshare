//! Visitors to collect various items from the AST.
use crate::{
    parser::{
        has_typeshare_annotation, parse_enum, parse_struct, parse_type_alias, ErrorInfo,
        ParseError, ParsedData,
    },
    rust_types::RustItem,
};
use syn::{visit::Visit, ItemUse, UseTree};

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

/// An import visitor that collects all use or
/// qualified referenced items.
#[derive(Default)]
pub struct TypeShareVisitor {
    parsed_data: ParsedData,
}

impl TypeShareVisitor {
    /// Create an import visitor for a given crate name.
    pub fn new(crate_name: String, file_name: String) -> Self {
        Self {
            parsed_data: ParsedData::new(crate_name, file_name),
        }
    }

    /// Consume the visitor and return parsed data.
    pub fn parsed_data(self) -> ParsedData {
        self.parsed_data
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
}

impl<'ast> Visit<'ast> for TypeShareVisitor {
    /// Find any reference types that are not part of
    /// the `use` import statements.
    fn visit_path(&mut self, p: &'ast syn::Path) {
        fn extract_root_and_types(p: &syn::Path, crate_name: &str) -> Option<ImportedType> {
            let first = p.segments.first()?.ident.to_string();
            let last = p.segments.last()?.ident.to_string();

            accept_crate(&first)
                .then_some(())
                .and_then(|_| accept_type(&last).then_some(()))?;

            (first != last).then(|| {
                // resolve crate and super aliases into the crate name.
                let base_crate = if first == "crate" || first == "super" {
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
            self.parsed_data.import_types.push(imported_type);
        }
        syn::visit::visit_path(self, p);
    }

    /// Collect referenced imports.
    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        let result = parse_import(i, &self.parsed_data.crate_name);
        self.parsed_data.import_types.extend(result);
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
    !IGNORED_BASE_CRATES.iter().any(|&n| n == crate_name)
        && crate_name
            .chars()
            .next()
            .map(|c| c.is_lowercase())
            .unwrap_or(false)
}

/// Accept types which start with an uppercase character.
fn accept_type(type_name: &str) -> bool {
    type_name
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

/// An imported type reference.
#[derive(Debug)]
pub struct ImportedType {
    /// Crate this type belongs to.
    pub base_crate: String,
    /// Type name.
    pub type_name: String,
}

/// Parse the base crate name and types from a `use` import statement.
fn parse_import(item_use: &ItemUse, crate_name: &str) -> Vec<ImportedType> {
    let mut names = Vec::new();
    let mut imported_types = Vec::new();

    fn traverse(
        use_tree: &UseTree,
        names: &mut Vec<String>,
        imported_types: &mut Vec<ImportedType>,
        crate_name: &str,
    ) {
        // resolve crate and super aliases into the crate name.
        let resolve_crate_name = || -> String {
            if names[0].as_str() == "crate" || names[0].as_str() == "super" {
                crate_name.to_string()
            } else {
                names[0].clone()
            }
        };

        match use_tree {
            syn::UseTree::Path(path) => {
                names.push(path.ident.to_string());
                traverse(&path.tree, names, imported_types, crate_name);
            }
            syn::UseTree::Name(name) => {
                let type_name = name.ident.to_string();
                if accept_crate(&names[0]) && accept_type(&type_name) {
                    imported_types.push(ImportedType {
                        base_crate: resolve_crate_name(),
                        type_name,
                    });
                }
            }
            syn::UseTree::Rename(rename) => {
                // TODO: I need to do something here.
                names.push(rename.ident.to_string());
            }
            syn::UseTree::Glob(_) => {
                if accept_crate(&names[0]) {
                    imported_types.push(ImportedType {
                        base_crate: resolve_crate_name(),
                        type_name: "*".into(),
                    });
                }
            }
            syn::UseTree::Group(g) => {
                g.items.iter().for_each(|item| {
                    traverse(item, names, imported_types, crate_name);
                });
            }
        }
    }

    traverse(&item_use.tree, &mut names, &mut imported_types, crate_name);
    imported_types
}

#[cfg(test)]
mod test {
    use super::{parse_import, TypeShareVisitor};
    use crate::visitors::ImportedType;
    use cool_asserts::assert_matches;
    use syn::{visit::Visit, File};

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
            use super::some_module::{another_module::MyType, MyEnum};
        ";
        let file = syn::parse_str::<syn::File>(rust_file).unwrap();

        let parsed_imports = file
            .items
            .iter()
            .flat_map(|item| {
                if let syn::Item::Use(use_item) = item {
                    parse_import(use_item, "my_crate")
                } else {
                    Vec::new()
                }
            })
            .collect::<Vec<_>>();

        assert_matches!(
            parsed_imports,
            [
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "quote");
                    assert_eq!(type_name, "ToTokens");
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
            use super::some_module::{another_module::MyType, MyEnum};

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
        let mut visitor = TypeShareVisitor::new("my_crate".into(), "my_file".into());
        visitor.visit_file(&file);

        assert_matches!(
            visitor.parsed_data.import_types,
            [
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "quote");
                    assert_eq!(type_name, "ToTokens");
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
                }  => {
                    assert_eq!(base_crate, "some_crate");
                    assert_eq!(type_name, "Type");
                },
                ImportedType {
                    base_crate,
                    type_name,
                }  => {
                    assert_eq!(base_crate, "my_crate");
                    assert_eq!(type_name, "TypeName");
                },
            ]
        );
    }
}
