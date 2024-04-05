//! Visitors to collect various items from the AST.
use syn::{visit::Visit, ItemUse, UseTree};

/// An import visitor that collects all use or
/// qualified referenced items.
#[derive(Default)]
pub struct ImportVisitor<'a> {
    pub import_types: Vec<ImportedType>,
    pub crate_name: &'a str,
}

impl<'a> ImportVisitor<'a> {
    pub fn new(crate_name: &'a str) -> Self {
        Self {
            crate_name,
            import_types: Vec::new(),
        }
    }
}

impl<'ast, 'a> Visit<'ast> for ImportVisitor<'a> {
    fn visit_path(&mut self, p: &'ast syn::Path) {
        fn extract_root_and_types(p: &syn::Path, crate_name: &str) -> Option<ImportedType> {
            let first = p.segments.first()?.ident.to_string();
            let last = p.segments.last()?.ident.to_string();
            (first != last).then(|| {
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

        if let Some(imported_type) = extract_root_and_types(p, self.crate_name) {
            self.import_types.push(imported_type);
        }
        syn::visit::visit_path(self, p);
    }

    fn visit_item_use(&mut self, i: &'ast ItemUse) {
        let result = parse_import(i, self.crate_name);
        self.import_types.extend(result);
        syn::visit::visit_item_use(self, i);
    }
}

#[derive(Debug)]
pub struct ImportedType {
    pub base_crate: String,
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
        match use_tree {
            syn::UseTree::Path(path) => {
                names.push(path.ident.to_string());
                traverse(&path.tree, names, imported_types, crate_name);
            }
            syn::UseTree::Name(name) => {
                imported_types.push(ImportedType {
                    // any imports from the same crate will be converted to
                    // the create name.
                    base_crate: if names[0].as_str() == "crate" || names[0].as_str() == "super" {
                        crate_name.to_string()
                    } else {
                        names[0].clone()
                    },
                    type_name: name.ident.to_string(),
                });
            }
            syn::UseTree::Rename(rename) => {
                names.push(rename.ident.to_string());
            }
            syn::UseTree::Glob(_) => imported_types.push(ImportedType {
                base_crate: names[0].clone(),
                type_name: "*".into(),
            }),
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
    use super::{parse_import, ImportVisitor};
    use crate::visitors::ImportedType;
    use cool_asserts::assert_matches;
    use syn::{visit::Visit, File};

    #[test]
    fn test_parse_import() {
        let rust_file = "
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
                    assert_eq!(base_crate, "std");
                    assert_eq!(type_name, "BTreeSet");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "std");
                    assert_eq!(type_name, "FromStr");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "std");
                    assert_eq!(type_name, "HashMap");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "std");
                    assert_eq!(type_name, "TryFrom");
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
        let mut visitor = ImportVisitor::new("my_crate");
        visitor.visit_file(&file);

        assert_matches!(
            visitor.import_types,
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
                    assert_eq!(base_crate, "std");
                    assert_eq!(type_name, "BTreeSet");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "std");
                    assert_eq!(type_name, "FromStr");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "std");
                    assert_eq!(type_name, "HashMap");
                },
                ImportedType {
                    base_crate,
                    type_name,
                } => {
                    assert_eq!(base_crate, "std");
                    assert_eq!(type_name, "TryFrom");
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
