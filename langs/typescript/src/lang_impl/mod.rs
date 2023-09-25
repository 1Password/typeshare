mod enum_generator;
mod struct_generator;
mod type_alias_generator;
mod types;

use std::io::Write;

use crate::config::TypeScriptConfig;
use thiserror::Error;
use typeshare_core::{
    language::{Generate, LangResult, Language, LanguageError, TypeFormatter, WriteTypesResult},
    parsed_types::{Comment, CommentLocation, Field, Item, ParsedData},
    topsort,
};

/// All information needed to generate Typescript type-code
#[derive(Default)]
pub struct TypeScript {
    pub config: TypeScriptConfig,
}
pub type TypescriptResult<T> = Result<T, LanguageError<TypescriptError>>;
#[derive(Error, Debug)]
pub enum TypescriptError {
    #[error(
        r#"
    Please give an explicit output type for 64 bit integer types.
    Or set the `use_bigint` option to true in your config.
    "#
    )]
    No64BitIntegerType,
    #[error("Generic key forbidden in typescript: {0}")]
    GenericKeyForbiddenInTS(String),
}
impl From<TypescriptError> for LanguageError<TypescriptError> {
    fn from(e: TypescriptError) -> Self {
        LanguageError::LanguageError(e)
    }
}

impl Language for TypeScript {
    type Config = TypeScriptConfig;
    type Error = TypescriptError;

    fn language_name() -> &'static str
    where
        Self: Sized,
    {
        "typescript"
    }

    fn extension(&self) -> &'static str {
        "ts"
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }

    fn generate_from_parse(
        &mut self,
        data: &ParsedData,
    ) -> LangResult<WriteTypesResult, Self::Error> {
        let mut file: Vec<u8> = Vec::new();

        let mut items: Vec<Item> = vec![];

        for a in &data.aliases {
            items.push(Item::Alias(a.clone()))
        }

        for s in &data.structs {
            items.push(Item::Struct(s.clone()))
        }

        for e in &data.enums {
            items.push(Item::Enum(e.clone()))
        }

        let sorted = topsort(items.iter().collect());

        for &thing in &sorted {
            match thing {
                Item::Enum(e) => e.generate_to(self, &mut file)?,
                Item::Struct(s) => s.generate_to(self, &mut file)?,
                Item::Alias(a) => a.generate_to(self, &mut file)?,
                _ => {}
            }
        }

        Ok(WriteTypesResult::SingleFile(String::from_utf8(file)?))
    }
}
impl Generate<TypeScript> for Comment {
    fn generate_to(&self, _: &mut TypeScript, write: &mut impl Write) -> TypescriptResult<()> {
        if !self.is_empty() {
            let tab_indent = match self.get_location() {
                CommentLocation::FileHeader => String::new(),
                CommentLocation::Type => String::new(),
                CommentLocation::Field => "\t".to_owned(),
            };
            let comment: String = match self {
                Comment::Single { comment, .. } => {
                    format!("{}/** {} */", tab_indent, comment)
                }
                Comment::Multiline { comment, .. } => {
                    let joined_comments = comment.join(&format!("\n{} * ", tab_indent));
                    format!(
                        "{tab}/**
                         {tab} * {comment}
                         {tab} */",
                        tab = tab_indent,
                        comment = joined_comments
                    )
                }
                Comment::None { .. } => {
                    unreachable!("Is empty returns false for None")
                }
            };

            writeln!(write, "{}", comment)?;
        }
        Ok(())
    }
}
impl TypeScript {
    fn write_field(
        &mut self,
        w: &mut impl Write,
        field: &Field,
        generic_types: &[String],
    ) -> TypescriptResult<()> {
        if field.comments.is_empty() {
            if let Some(comments) = self.config.type_mappings.get_comments(field.ty.id()) {
                let comments = comments.clone();
                comments.generate_to(self, w)?;
            }
        } else {
            field.comments.generate_to(self, w)?;
        }
        let ts_ty: String = match field.type_override("typescript") {
            Some(type_override) => type_override.to_owned(),
            None => self.format_type(&field.ty, generic_types)?,
        };

        let optional = field.ty.is_optional() || field.has_default;
        let double_optional = field.ty.is_double_optional();
        let is_readonly = field
            .lang_decorators
            .get("typescript")
            .filter(|v| v.iter().any(|dec| dec.name() == "readonly"))
            .is_some();
        writeln!(
            w,
            "\t{}{}{}: {}{};",
            is_readonly.then(|| "readonly ").unwrap_or_default(),
            typescript_property_aware_rename(&field.id.renamed),
            optional.then(|| "?").unwrap_or_default(),
            ts_ty,
            double_optional.then(|| " | null").unwrap_or_default()
        )?;

        Ok(())
    }
}

fn typescript_property_aware_rename(name: &str) -> String {
    if name.chars().any(|c| c == '-') {
        return format!("{:?}", name);
    }
    name.to_string()
}
