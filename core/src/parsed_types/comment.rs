use std::fmt::Display;

use strum::{Display, EnumIs, EnumIter, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs, Display, IntoStaticStr, EnumIter)]
#[cfg_attr(
    feature = "serde-everything",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum CommentLocation {
    FileHeader,
    Type,
    Field,
}

#[derive(Debug, PartialEq, Eq, Clone, EnumIs)]
pub enum Comment {
    Single {
        /// The comment
        comment: String,
        /// The location of the comment
        location: CommentLocation,
    },
    Multiline {
        /// The comment
        comment: Vec<String>,
        /// The location of the comment
        location: CommentLocation,
    },
    None {
        location: CommentLocation,
    },
}
impl From<Vec<String>> for Comment {
    fn from(comment: Vec<String>) -> Self {
        if comment.len() == 1 {
            Comment::Single {
                comment: comment.into_iter().next().unwrap(),
                location: CommentLocation::Type,
            }
        } else {
            Comment::Multiline {
                comment,
                location: CommentLocation::Type,
            }
        }
    }
}

impl Default for Comment {
    fn default() -> Self {
        Comment::None {
            location: CommentLocation::Type,
        }
    }
}

impl AsRef<CommentLocation> for Comment {
    fn as_ref(&self) -> &CommentLocation {
        match self {
            Comment::Single { location, .. } => location,
            Comment::Multiline { location, .. } => location,
            Comment::None { location, .. } => location,
        }
    }
}

impl Comment {
    pub fn new_single(comment: impl Into<String>, location: CommentLocation) -> Self {
        Comment::Single {
            comment: comment.into(),
            location,
        }
    }

    pub fn new_multiline(comment: impl Into<Vec<String>>, location: CommentLocation) -> Self {
        Comment::Multiline {
            comment: comment.into(),
            location,
        }
    }

    pub fn get_location(&self) -> CommentLocation {
        match self {
            Comment::Single { location, .. } => *location,
            Comment::Multiline { location, .. } => *location,
            Comment::None { location } => *location,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Comment::Single { comment, .. } => comment.is_empty(),
            Comment::Multiline { comment, .. } => comment.is_empty(),
            _ => true,
        }
    }
    pub fn first(&self) -> Option<&str> {
        match self {
            Comment::Single { comment, .. } => Some(comment.as_str()),
            Comment::Multiline { comment, .. } => comment.first().map(|s| s.as_str()),
            _ => None,
        }
    }
    pub fn set_location(&mut self, location: CommentLocation) {
        match self {
            Comment::Single { location: l, .. } => *l = location,
            Comment::Multiline { location: l, .. } => *l = location,
            Comment::None { location: l } => *l = location,
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Comment::Single { .. } => 1,
            Comment::Multiline { comment, .. } => comment.len(),
            _ => 0,
        }
    }
}
impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comment::Single { comment, .. } => writeln!(f, "{}", comment),
            Comment::Multiline { comment, .. } => {
                let mut iter = comment.iter().peekable();
                while let Some(line) = iter.next() {
                    write!(f, "{}", line)?;
                    if iter.peek().is_some() {
                        write!(f, "\n")?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
mod impl_serde {
    use std::fmt::Formatter;

    use serde::{
        de::{Error, SeqAccess, Visitor},
        ser::SerializeSeq,
        Deserialize, Serialize,
    };

    use super::Comment;
    use crate::parsed_types::comment::CommentLocation;

    impl Serialize for Comment {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Comment::Single { comment, .. } => serializer.serialize_str(comment),
                Comment::Multiline { comment, .. } => {
                    let mut seq = serializer.serialize_seq(Some(comment.len()))?;
                    for element in comment {
                        seq.serialize_element(element)?;
                    }
                    seq.end()
                }
                Comment::None { .. } => serializer.serialize_none(),
            }
        }
    }
    struct CommentVisitor;
    impl<'de> Visitor<'de> for CommentVisitor {
        type Value = Comment;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a sequence of strings")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Comment::Single {
                comment: v.to_string(),
                location: CommentLocation::Type,
            })
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Comment::Single {
                comment: v,
                location: CommentLocation::Type,
            })
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut comment = Vec::new();
            while let Some(s) = seq.next_element()? {
                comment.push(s);
            }
            Ok(Comment::Multiline {
                comment,
                location: CommentLocation::Type,
            })
        }
    }
    impl<'de> Deserialize<'de> for Comment {
        fn deserialize<D>(deserializer: D) -> Result<Comment, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(CommentVisitor)
        }
    }
}
