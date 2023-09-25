use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::Path};

use crate::rename::RenameAll;

/// Identifier used in Rust structs, enums, and fields. It includes the `original` name and the `renamed` value after the transformation based on `serde` attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Id {
    /// The original identifier name
    pub original: String,
    /// The renamed identifier, based on serde attributes.
    /// If there is no re-naming going on, this will be identical to
    /// `original`.
    pub renamed: String,

    pub rename_all: Option<RenameAll>,
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.original == self.renamed {
            write!(f, "({})", self.original)
        } else {
            write!(f, "({}, {})", self.original, self.renamed)
        }
    }
}
/// The source of the Rust Item
///
/// This does not guarantee that it is completely accurate, but it is the best we can do.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(tag = "type", content = "value")]
pub enum Source {
    Rust { crate_name: String, path: String },
    Other { path: String },
    Inline,
}
impl Default for Source {
    fn default() -> Self {
        Self::Inline
    }
}
impl Source {
    pub fn new_rust(crate_name: &str) -> Self {
        Self::Rust {
            crate_name: crate_name.to_string(),
            path: String::default(),
        }
    }

    pub(crate) fn build_from_path(&mut self, path: impl AsRef<Path>) {
        match self {
            Source::Rust { path: og_path, .. } => {
                let path = path.as_ref();
                let path = path.iter();

                *og_path = path
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join("::");
            }
            Source::Other { path: old_path } => {
                *old_path = path.as_ref().to_string_lossy().to_string();
            }
            Source::Inline => {
                *self = Source::Other {
                    path: path.as_ref().to_string_lossy().to_string(),
                }
            }
        }
    }
    pub fn push(&self, path: &str) -> Source {
        let mut source = self.clone();
        match &mut source {
            Source::Rust {
                path: rust_path, ..
            } => {
                if rust_path.is_empty() {
                    *rust_path = path.to_string();
                } else {
                    rust_path.push_str("::");
                    rust_path.push_str(path);
                }
                source
            }
            Source::Other { .. } => {
                source.build_from_path(path);
                source
            }
            Source::Inline => {
                source.build_from_path(path);
                source
            }
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Rust { path, crate_name } => {
                if path.is_empty() {
                    write!(f, "{}", crate_name)
                } else {
                    write!(f, "{}::{}", crate_name, path)
                }
            }
            Source::Other { path } => {
                write!(f, "{}", path)
            }
            Source::Inline => write!(f, "Inline"),
        }
    }
}
