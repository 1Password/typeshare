//! Optional checks for `#[cfg(target_os = "target")]
use crate::parser::get_meta_items;
use log::{debug, error, log_enabled, warn};
use quote::ToTokens;
use syn::{punctuated::Punctuated, Attribute, Expr, ExprLit, Lit, Meta, Token};

#[derive(Copy, Clone, Default, Debug)]
/// Scoped inside a block that either accepts
/// or rejects.
enum TargetScope {
    #[default]
    Accept,
    Reject,
}

#[derive(Default)]
/// An iterator that yields all meta items and their contained scope.
struct TargetOsIterator {
    meta: Vec<(TargetScope, Meta)>,
}

impl TargetOsIterator {
    /// Create a new nested meta iterator.
    fn new(meta: Meta) -> Self {
        Self {
            meta: Vec::from([(TargetScope::Accept, meta)]),
        }
    }
}

impl Iterator for TargetOsIterator {
    type Item = (TargetScope, String);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((mut scope, meta)) = self.meta.pop() {
            if meta.path().is_ident("not") {
                debug!("encountered not");
                scope = TargetScope::Reject
            }

            match meta {
                Meta::Path(p) => {
                    if log_enabled!(log::Level::Warn) {
                        warn!(
                            "Encountered path while traversing target_os candidates: {}",
                            p.into_token_stream()
                        );
                    }
                }
                Meta::List(meta_list) => {
                    let nested_meta_list = meta_list
                        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                        .inspect_err(|err| {
                            error!("Failed to parse nested meta while traversing target_os candidates: {err}");
                        })
                        .ok()?;
                    debug!("\texpanding with {} meta", nested_meta_list.len());
                    self.meta
                        .extend(nested_meta_list.into_iter().map(|meta| (scope, meta)));
                }
                Meta::NameValue(nv) => {
                    #[cfg(test)]
                    debug!("\tworking with NameValue: {nv:?}");
                    if let Some(value) =
                        nv.path
                            .is_ident("target_os")
                            .then_some(nv.value)
                            .and_then(|value| match value {
                                Expr::Lit(ExprLit {
                                    lit: Lit::Str(val), ..
                                }) => Some(val.value()),
                                _ => None,
                            })
                    {
                        return Some((scope, value));
                    }
                }
            }
        }
        None
    }
}

pub(crate) fn accept_target_os(attrs: &[Attribute], target_os: &[String]) -> bool {
    if target_os.is_empty() {
        return true;
    }

    let (accepted, rejected): (Vec<_>, Vec<_>) = attrs
        .iter()
        .inspect(|attr| {
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "\tchecking attribute {} for {target_os:?} accept",
                    attr.into_token_stream()
                );
            }
        })
        .flat_map(|attr| get_meta_items(attr, "cfg"))
        .flat_map(TargetOsIterator::new)
        .inspect(|val| debug!("Yielded {val:?}"))
        .partition(|(scope, _)| match scope {
            TargetScope::Accept => true,
            TargetScope::Reject => false,
        });

    debug!("accepted: {accepted:?}, rejected: {rejected:?}");

    let is_rejected = || {
        target_os
            .iter()
            .any(|target| rejected.iter().any(|(_, rejected)| target == rejected))
    };

    let is_accepted = || {
        accepted.is_empty()
            || target_os
                .iter()
                .any(|target| accepted.iter().any(|(_, accepted)| accepted == target))
    };

    !is_rejected() && is_accepted()
}

#[cfg(test)]
mod test {
    use super::accept_target_os;
    use flexi_logger::DeferredNow;
    use log::Record;
    use std::{io::Write, sync::Once};
    use syn::{parse_quote, ItemEnum, ItemStruct};

    static INIT: Once = Once::new();

    fn init_log() {
        INIT.call_once(|| {
            flexi_logger::Logger::try_with_env()
                .unwrap()
                .format(
                    |write: &mut dyn Write, _now: &mut DeferredNow, record: &Record<'_>| {
                        let file_name = record.file().unwrap_or_default();
                        let file_name = if file_name.len() > 15 {
                            let split = file_name.len() - 15;
                            &file_name[split..]
                        } else {
                            file_name
                        };
                        write!(
                            write,
                            "{file_name:>15}{:>5} - {}",
                            record.line().unwrap_or_default(),
                            record.args()
                        )
                    },
                )
                .start()
                .unwrap();
        })
    }

    #[test]
    fn test_target_os_nested_reject() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(all(feature = "my-feature", not(target_os = "ios")))]
            pub struct NestedNotTarget;
        };

        assert!(!accept_target_os(
            &test_struct.attrs,
            &["ios".into(), "android".into()]
        ));
    }

    #[test]
    fn test_target_os_accept() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(target_os = "android")]
            pub struct NestedNotTarget;
        };

        assert!(accept_target_os(
            &test_struct.attrs,
            &["ios".into(), "android".into()]
        ));
    }

    #[test]
    fn test_target_os_combined_any_accepted() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(any(target_os = "android", target_os = "ios"))]
            pub struct NestedNotTarget;
        };

        assert!(accept_target_os(
            &test_struct.attrs,
            &["ios".into(), "android".into()]
        ));
    }

    #[test]
    fn test_target_os_combined_all_accepted() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(all(target_os = "windows", target_os = "android"))]
            pub struct NestedNotTarget;
        };

        assert!(accept_target_os(
            &test_struct.attrs,
            &["ios".into(), "android".into()]
        ));
    }

    #[test]
    fn test_target_os_combined_rejected() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(not(any(target_os = "wasm32", target_os = "ios")))]
            pub struct NestedNotTarget;
        };

        assert!(!accept_target_os(
            &test_struct.attrs,
            &["ios".into(), "android".into()]
        ));
    }

    #[test]
    fn test_accept_no_target_os() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(feature = "my-feature")]
            pub struct NestedNotTarget;
        };

        assert!(accept_target_os(
            &test_struct.attrs,
            &["ios".into(), "android".into()]
        ));
    }

    #[test]
    fn test_accept_no_attribute() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            pub struct NestedNotTarget;
        };

        assert!(accept_target_os(
            &test_struct.attrs,
            &["ios".into(), "android".into()]
        ));
    }

    #[test]
    fn test_accept_none_excluded() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(not(any(target_os = "wasm32", target_os = "ios")))]
            pub struct Excluded;
        };

        assert!(accept_target_os(
            &test_struct.attrs,
            &["macos".into(), "android".into()]
        ));
    }

    #[test]
    fn test_reject_not_target_os() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(target_os = "ios")]
            pub struct Excluded;
        };

        assert!(!accept_target_os(
            &test_struct.attrs,
            &["macos".into(), "android".into()]
        ));
    }

    #[test]
    fn test_any_target_not_target_os() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(any(target_os = "ios", feature = "test"))]
            pub struct Excluded;
        };

        assert!(!accept_target_os(
            &test_struct.attrs,
            &["macos".into(), "android".into()]
        ));
    }

    #[test]
    fn test_not_scope() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(all(not(feature = "my-feature"), target_os = "android"))]
            pub struct Test;
        };

        assert!(accept_target_os(&test_struct.attrs, &["android".into()]))
    }

    #[test]
    fn test_not_scope_reverse() {
        init_log();

        let test_struct: ItemStruct = parse_quote! {
            #[cfg(all(target_os = "android", not(feature = "my-feature")))]
            pub struct Test;
        };

        assert!(accept_target_os(&test_struct.attrs, &["android".into()]))
    }

    #[test]
    fn test_enum_no_target_os_enabled() {
        init_log();

        let test_enum: ItemEnum = parse_quote! {
            #[typeshare]
            pub enum TestEnum {
                #[cfg(target_os = "ios")]
                Variant1,
                #[cfg(target_os = "android")]
                Variant2,
            }
        };

        let variants = test_enum
            .variants
            .iter()
            .map(|v| accept_target_os(&v.attrs, &[]))
            .collect::<Vec<_>>();

        assert_eq!(&variants, &[true, true]);
    }
}
