use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Generics {
    pub generics: Vec<String>,
}
impl From<Vec<String>> for Generics {
    fn from(generics: Vec<String>) -> Self {
        Self { generics }
    }
}
impl Deref for Generics {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.generics
    }
}
