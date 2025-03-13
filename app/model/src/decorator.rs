use std::collections::HashMap;

/// A decorator value can either be any literal (`ident = "foo"`), or absent
/// entirely (`ident`).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    None,
    Bool(bool),
    String(String),
    Int(u32),
    // Don't add anything here that isn't a literal, but feel free to add new
    // literals as support for them is desired
}

pub type DecoratorSet = HashMap<String, Vec<Value>>;
