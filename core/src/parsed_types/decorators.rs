use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Decorators {
    pub(crate) decorators: Vec<Decorator>,
}

impl Into<Decorators> for Vec<Decorator> {
    fn into(self) -> Decorators {
        Decorators { decorators: self }
    }
}
impl Deref for Decorators {
    type Target = Vec<Decorator>;

    fn deref(&self) -> &Self::Target {
        &self.decorators
    }
}

impl DerefMut for Decorators {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.decorators
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde-everything", serde(tag = "type", content = "value"))]
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
