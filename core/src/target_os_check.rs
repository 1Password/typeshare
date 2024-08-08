//! Optional checks for `#[cfg(target_os = "target")]
use crate::parser::get_meta_items;
use log::{debug, error, log_enabled};
use quote::ToTokens;
use std::collections::VecDeque;
use syn::{punctuated::Punctuated, Attribute, Expr, ExprLit, Lit, Meta, Token};

#[derive(Copy, Clone, Default, Debug)]
enum TargetScope {
    #[default]
    Accept,
    Reject,
}

#[derive(Default)]
struct TargetOsIterator {
    meta: VecDeque<Meta>,
    scope: TargetScope,
}

impl TargetOsIterator {
    fn new(meta: Meta) -> Self {
        Self {
            meta: VecDeque::from([meta]),
            ..Default::default()
        }
    }
}

impl Iterator for TargetOsIterator {
    type Item = (TargetScope, String);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(meta) = self.meta.pop_front() {
            debug!("working on meta");
            if meta.path().is_ident("not") {
                debug!("encountered not");
                self.scope = TargetScope::Reject
            }

            match meta {
                Meta::Path(p) => {
                    debug!("\tencountered path: {p:?}");
                }
                Meta::List(meta_list) => {
                    let nested_meta_list = meta_list
                        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                        .inspect_err(|err| {
                            error!("\tfailed to parse nested meta: {err}");
                        })
                        .ok()?;
                    debug!("\texpanding with {} meta", nested_meta_list.len());
                    self.meta.extend(nested_meta_list);
                }
                Meta::NameValue(nv) => {
                    debug!("\tworking on NameValue: {nv:?}");
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
                        return Some((self.scope, value));
                    }
                }
            }
        }
        None
    }
}

pub(crate) fn accept_target_os(attrs: &[Attribute], target_os: &[String]) -> bool {
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
    use syn::{parse_quote, ItemStruct};

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
}
