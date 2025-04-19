use std::collections::HashMap;

/// A decorator value can either be any literal (`ident = "foo"`), or absent
/// entirely (`ident`).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// The key was present as an attribute without an associated value.
    None,
    Bool(bool),
    String(String),
    Int(u32),
    Nested(DecoratorSet),
}

/// A set of decorators attached to something via the `#[typeshare]` attribute.
/// Decorators can appear more than once, so each key includes one more values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DecoratorSet {
    set: HashMap<String, Vec<Value>>,
}

/// Basic data structure methods
impl DecoratorSet {
    /// Create a new, empty `DecoratorSet`
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new decorator to this set.
    pub fn add(&mut self, key: String, value: Value) {
        self.set.entry(key).or_default().push(value);
    }

    /// Get all of the decorators associated with a given key, in insertion
    /// order.
    pub fn get_all(&self, key: &str) -> &[Value] {
        self.set
            .get(key)
            .map(|values| values.as_slice())
            .unwrap_or(&[])
    }

    /// Get the *first* decorator associated with a given key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.set.get(key)?.first()
    }

    /// Test if any decorator for this key matches the given test
    pub fn any(&self, key: &str, test: impl FnMut(&Value) -> bool) -> bool {
        self.get_all(key).iter().any(test)
    }
}

impl Extend<(String, Value)> for DecoratorSet {
    fn extend<T: IntoIterator<Item = (String, Value)>>(&mut self, iter: T) {
        iter.into_iter()
            .for_each(|(key, value)| self.add(key, value));
    }
}

impl<T> FromIterator<T> for DecoratorSet
where
    DecoratorSet: Extend<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Self::new();
        set.extend(iter);
        set
    }
}

/// Convenience methods for common decorators (especially ones not specific
/// to particular languages)
impl DecoratorSet {
    /// `#[typeshare(redacted)`
    pub fn is_redacted(&self) -> bool {
        self.any("redacted", |value| matches!(*value, Value::None))
    }

    /// Languages can include lang-specific type overrides like this:
    ///
    /// `#[typeshare(swift(type = "string"), kotlin(type = "STRING"))]`
    ///
    /// We assume that a language's type override should be a string.
    ///
    /// Note that this is just a convention; languages have to explicitly
    /// check for this type override.
    pub fn type_override_for_lang(&self, lang: &str) -> Option<&str> {
        self.get_all(lang)
            .iter()
            .filter_map(|value| match value {
                Value::Nested(set) => Some(set),
                _ => None,
            })
            .flat_map(|set| set.get_all("type"))
            .find_map(|value| match value {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            })
    }
}
