use proc_macro2::{Delimiter, Group, TokenStream};
use syn::{parse::Parser, punctuated::Punctuated, Ident, LitStr, Token};

/// A single predicate, like `test` or `feature = "foo"` or `target_os = "ios"`
pub struct Predicate {
    pub key: Ident,
    pub value: Option<String>,
}

/// The outcome of a 3-state logical operation. We need this because we
/// need to be able to accept both `feature = "foo"` and `not(feature = "foo")`,
/// because we don't actually care what features are configured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    False,
    Maybe,
    True,
}

/// Anything that can be inside a `cfg!( ... )`
impl Outcome {
    pub fn not(self) -> Self {
        match self {
            Outcome::False => Outcome::True,
            Outcome::Maybe => Outcome::Maybe,
            Outcome::True => Outcome::False,
        }
    }

    pub fn and(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Outcome::False, _) | (_, Outcome::False) => Outcome::False,
            (Outcome::Maybe, _) | (_, Outcome::Maybe) => Outcome::Maybe,
            (Outcome::True, Outcome::True) => Outcome::True,
        }
    }

    pub fn or(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Outcome::True, _) | (_, Outcome::True) => Outcome::True,
            (Outcome::Maybe, _) | (_, Outcome::Maybe) => Outcome::Maybe,
            (Outcome::False, Outcome::False) => Outcome::False,
        }
    }
}

pub enum Cfg {
    Predicate(Predicate),
    All(Vec<Cfg>),
    Any(Vec<Cfg>),
    Not(Box<Cfg>),
}

impl Cfg {
    pub fn test(&self, func: &impl Fn(&Predicate) -> Outcome) -> Outcome {
        match self {
            Cfg::Predicate(predicate) => func(predicate),
            Cfg::All(cfgs) => cfgs
                .iter()
                .fold(Outcome::True, |outcome, cfg| outcome.and(cfg.test(func))),
            Cfg::Any(cfgs) => cfgs
                .iter()
                .fold(Outcome::False, |outcome, cfg| outcome.or(cfg.test(func))),
            Cfg::Not(cfg) => cfg.test(func).not(),
        }
    }
}

fn get_paren_group(input: syn::parse::ParseStream) -> syn::Result<TokenStream> {
    let group: Group = input.parse()?;

    let Delimiter::Parenthesis = group.delimiter() else {
        return Err(syn::Error::new(
            group.span(),
            "expected parenthesized group",
        ));
    };

    Ok(group.stream())
}

impl syn::parse::Parse for Cfg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let token: Ident = input.parse()?;

        match token.to_string().as_str() {
            "all" => {
                let group = get_paren_group(input)?;
                let predicates: Punctuated<Cfg, Token![,]> =
                    Punctuated::parse_terminated.parse2(group)?;
                Ok(Self::All(predicates.into_iter().collect()))
            }
            "any" => {
                let group = get_paren_group(input)?;
                let predicates: Punctuated<Cfg, Token![,]> =
                    Punctuated::parse_terminated.parse2(group)?;
                Ok(Self::Any(predicates.into_iter().collect()))
            }
            "not" => {
                let group = get_paren_group(input)?;
                let predicate = Cfg::parse.parse2(group)?;
                Ok(Self::Not(Box::new(predicate)))
            }
            _ => {
                let tok: Option<Token![=]> = input.parse()?;
                let value: Option<LitStr> = tok.map(|_tok| input.parse()).transpose()?;
                let value = value.map(|value| value.value());
                Ok(Self::Predicate(Predicate { key: token, value }))
            }
        }
    }
}

/// Check if the target os is okay. This method returns false only when the
/// OS was explicitly rejected.
///
/// If multiple OSes are given, this will check them separately, and return
/// true if any are individually acceptable. This prevents things like
/// `all(os="a", os="b")`, which should never pass.
pub fn target_os_good(config: &Cfg, valid: &[&str]) -> bool {
    valid.iter().any(|valid| {
        let outcome = config.test(&|pred| {
            if pred.key == "target_os" {
                if let Some(value) = pred.value.as_deref() {
                    return match valid.contains(&value) {
                        true => Outcome::True,
                        false => Outcome::False,
                    };
                }
            }

            Outcome::Maybe
        });

        match outcome {
            Outcome::False => false,

            // If the outcome is Maybe, it means we didn't learn enough about
            // the OS. There wasn't a rejection, so we return true.
            Outcome::True | Outcome::Maybe => true,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    macro_rules! make_cfg {
        (
            all($(
                $key:ident $value:tt
            ),+ $(,)?)
        ) => {
            Cfg::All(
                Vec::from([$(
                    make_cfg!($key $value),
                )+])
            )
        };

        (
            any($(
                $key:ident $value:tt
            ),+ $(,)?)
        ) => {
            Cfg::Any(
                Vec::from([$(
                    make_cfg!($key $value),
                )+])
            )
        };

        (
            not($key:ident $value:tt)
        ) => {
            Cfg::Not(
                Box::new(
                    make_cfg!($key $value)
                )
            )
        };

        (
            $key:ident $value:literal
        ) => {
            Cfg::Predicate(
                Predicate {
                    key: Ident::new(stringify!($key), Span::call_site()),
                    value: Some($value.to_owned()),
                }
            )
        };
    }

    #[test]
    fn no_restriction() {
        let cfg = make_cfg!(all(feature "foo", feature "bar"));

        assert!(target_os_good(&cfg, &["windows"]));
    }

    #[test]
    fn no_restriction2() {
        let cfg = make_cfg!(not(all(feature "foo", feature "bar")));

        assert!(target_os_good(&cfg, &["windows"]));
    }

    #[test]
    fn test_basic() {
        let cfg = make_cfg!(target_os "linux");

        assert!(target_os_good(&cfg, &["linux"]));
        assert!(!target_os_good(&cfg, &["mac"]))
    }

    #[test]
    fn test_nested_reject() {
        let cfg = make_cfg!(
            all(feature "my-feature", not(target_os "ios"))
        );

        assert!(target_os_good(&cfg, &["linux"]));
        assert!(target_os_good(&cfg, &["windows"]));
        assert!(!target_os_good(&cfg, &["ios"]));
    }

    #[test]
    fn test_any() {
        let cfg = make_cfg!(
            any(target_os "windows", target_os "linux")
        );

        assert!(target_os_good(&cfg, &["linux"]));
        assert!(target_os_good(&cfg, &["windows"]));

        assert!(target_os_good(&cfg, &["linux", "mac"]));
        assert!(target_os_good(&cfg, &["windows", "mac"]));

        assert!(!target_os_good(&cfg, &["mac"]));
        assert!(!target_os_good(&cfg, &["mac", "ios"]))
    }

    #[test]
    fn test_all() {
        // It shouldn't ever be possible for this to succeed
        let cfg = make_cfg!(
            all(target_os "windows", target_os "android")
        );

        assert!(!target_os_good(&cfg, &["windows", "android"]));
        assert!(!target_os_good(&cfg, &["windows", "android", "mac"]));

        assert!(!target_os_good(&cfg, &["windows", "mac"]));
        assert!(!target_os_good(&cfg, &["android", "mac"]));

        assert!(!target_os_good(&cfg, &["windows"]));
        assert!(!target_os_good(&cfg, &["android"]));
    }

    #[test]
    fn test_reject_any() {
        let cfg = make_cfg!(
            not(any(target_os "windows", target_os "mac"))
        );

        assert!(target_os_good(&cfg, &["ios"]));
        assert!(target_os_good(&cfg, &["android"]));

        assert!(!target_os_good(&cfg, &["windows"]));
        assert!(!target_os_good(&cfg, &["windows"]));
    }

    #[test]
    fn test_all_with_feature() {
        let cfg = make_cfg!(
            all(feature "foo", any(target_os "windows", target_os "mac"))
        );

        assert!(target_os_good(&cfg, &["windows"]));
        assert!(target_os_good(&cfg, &["mac"]));

        assert!(target_os_good(&cfg, &["windows", "linux"]));
        assert!(target_os_good(&cfg, &["mac", "linux"]));

        assert!(!target_os_good(&cfg, &["linux"]))
    }
}
