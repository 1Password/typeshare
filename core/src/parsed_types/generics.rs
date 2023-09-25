use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]

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
