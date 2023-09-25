use derive_more::{Deref, DerefMut, From, Into};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(
    Debug, Clone, PartialEq, Eq, Default, From, Into, Deref, DerefMut, Serialize, Deserialize,
)]
pub struct Decorators {
    pub(crate) decorators: Vec<Decorator>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DecoratorsMap(pub(crate) HashMap<String, Decorators>);
impl DecoratorsMap {
    pub fn get(&self, language: &str) -> Option<&Decorators> {
        self.0.get(language)
    }

    pub fn get_mut(&mut self, language: &str) -> Option<&mut Decorators> {
        self.0.get_mut(language)
    }

    pub fn insert(&mut self, language: String, decorators: impl Into<Decorators>) {
        let decorators = decorators.into();
        if let Some(existing) = self.0.get_mut(&language) {
            existing.decorators.extend(decorators.decorators);
        } else {
            self.0.insert(language, decorators);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
#[non_exhaustive]
pub enum Decorator {
    ValueEquals { key: String, value: String },
    LangType(String),
    Word(String),
}
impl Decorator {
    pub fn name(&self) -> &str {
        match self {
            Decorator::ValueEquals { key, value: _ } => key,
            Decorator::LangType(_) => "type",
            Decorator::Word(name) => name,
        }
    }
}
