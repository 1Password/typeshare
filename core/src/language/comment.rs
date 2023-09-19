use serde::ser::SerializeSeq;
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::Display;
use strum::{Display, EnumIter, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, IntoStaticStr, EnumIter)]
pub enum CommentLocation {
    FileHeader,
    Type,
    Field,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Comment<'a> {
    Single {
        /// The comment
        comment: Cow<'a, str>,
        /// The location of the comment
        location: CommentLocation,
    },
    Multiline {
        /// The comment
        comment: &'a [&'a str],
        /// The location of the comment
        location: CommentLocation,
    },
    MultilineOwned {
        /// The comment
        comment: Vec<String>,
        /// The location of the comment
        location: CommentLocation,
    },
    None {
        location: CommentLocation,
    },
}
impl From<Vec<String>> for Comment<'_> {
    fn from(comment: Vec<String>) -> Self {
        Comment::MultilineOwned {
            comment,
            location: CommentLocation::Type,
        }
    }
}
impl Serialize for Comment<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Comment::Single { comment, .. } => serializer.serialize_str(comment),
            Comment::Multiline { comment, .. } => {
                let mut seq = serializer.serialize_seq(Some(comment.len()))?;
                for element in *comment {
                    seq.serialize_element(element)?;
                }
                seq.end()
            }
            Comment::MultilineOwned { comment, .. } => comment.serialize(serializer),
            Comment::None { .. } => serializer.serialize_none(),
        }
    }
}
impl Default for Comment<'_> {
    fn default() -> Self {
        Comment::None {
            location: CommentLocation::Type,
        }
    }
}

impl Clone for Comment<'_> {
    fn clone(&self) -> Self {
        match self {
            Comment::Single { comment, location } => Comment::Single {
                comment: comment.clone(),
                location: *location,
            },
            Comment::Multiline { comment, location } => Comment::MultilineOwned {
                comment: comment.iter().map(|s| s.to_string()).collect(),
                location: *location,
            },
            Comment::MultilineOwned { comment, location } => Comment::MultilineOwned {
                comment: comment.clone(),
                location: *location,
            },
            Comment::None { location } => Comment::None {
                location: *location,
            },
        }
    }
}

impl AsRef<CommentLocation> for Comment<'_> {
    fn as_ref(&self) -> &CommentLocation {
        match self {
            Comment::Single { location, .. } => location,
            Comment::Multiline { location, .. } => location,
            Comment::MultilineOwned { location, .. } => location,
            Comment::None { location, .. } => location,
        }
    }
}

impl<'a> Comment<'a> {
    pub fn new_single(comment: impl Into<Cow<'a, str>>, location: CommentLocation) -> Self {
        Comment::Single {
            comment: comment.into(),
            location,
        }
    }

    pub fn new_multiline(comment: &'a [&'a str], location: CommentLocation) -> Self {
        Comment::Multiline { comment, location }
    }
    fn write_multi_line(
        lines: &[impl AsRef<str>],
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for line in lines {
            writeln!(f, "{}", line.as_ref())?;
        }
        Ok(())
    }
    pub fn get_location(&self) -> CommentLocation {
        match self {
            Comment::Single { location, .. } => *location,
            Comment::Multiline { location, .. } => *location,
            Comment::MultilineOwned { location, .. } => *location,
            Comment::None { location } => *location,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Comment::Single { comment, .. } => comment.is_empty(),
            Comment::Multiline { comment, .. } => comment.is_empty(),
            Comment::MultilineOwned { comment, .. } => comment.is_empty(),
            _ => true,
        }
    }
    pub fn first(&self) -> Option<Cow<'_, str>> {
        match self {
            Comment::Single { comment, .. } => Some(comment.clone()),
            Comment::Multiline { comment, .. } => comment.first().map(|s| s.to_string().into()),
            Comment::MultilineOwned { comment, .. } => comment.first().map(|s| s.clone().into()),
            _ => None,
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Comment::Single { comment, .. } => 1,
            Comment::Multiline { comment, .. } => comment.len(),
            Comment::MultilineOwned { comment, .. } => comment.len(),
            _ => 0,
        }
    }
}
impl Display for Comment<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comment::Single { comment, .. } => writeln!(f, "{}", comment),
            Comment::Multiline { comment, .. } => Comment::write_multi_line(comment, f),
            Comment::MultilineOwned { comment, .. } => {
                Comment::write_multi_line(comment.as_slice(), f)
            }
            _ => Ok(()),
        }
    }
}
