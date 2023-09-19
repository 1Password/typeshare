use crate::{EnumWriteMethod, TypeScriptConfig};
use joinery::JoinableIterator;
use std::io::Write;
use thiserror::Error;
use typeshare_core::language::{Comment, CommentLocation, LangResult, Language, LanguageError};
use typeshare_core::rust_types::{
    RustEnum, RustEnumVariant, RustField, RustStruct, RustType, RustTypeAlias, RustTypeFormatError,
    SpecialRustType,
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
}
impl Into<LanguageError<TypescriptError>> for TypescriptError {
    fn into(self) -> LanguageError<TypescriptError> {
        LanguageError::LanguageError(self)
    }
}
impl Language for TypeScript {
    type Config = TypeScriptConfig;
    type Error = TypescriptError;

    fn language_name(&self) -> &'static str {
        "typescript"
    }

    fn extension(&self) -> &'static str {
        "ts"
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }
    fn format_special_type(
        &mut self,
        special_ty: &SpecialRustType,
        generic_types: &[String],
    ) -> TypescriptResult<String> {
        if let Some(type_override) = self.config.type_mappings.get(special_ty.id()) {
            return Ok(type_override.to_string());
        }
        match special_ty {
            SpecialRustType::Vec(rtype) => {
                Ok(format!("{}[]", self.format_type(rtype, generic_types)?))
            }
            SpecialRustType::Array(rtype, len) => {
                let formatted_type = self.format_type(rtype, generic_types)?;
                Ok(format!(
                    "[{}]",
                    std::iter::repeat(&formatted_type)
                        .take(*len)
                        .join_with(", ")
                ))
            }
            SpecialRustType::Slice(rtype) => {
                Ok(format!("{}[]", self.format_type(rtype, generic_types)?))
            }
            // We add optionality above the type formatting level
            SpecialRustType::Option(rtype) => self.format_type(rtype, generic_types),
            SpecialRustType::HashMap(rtype1, rtype2) => Ok(format!(
                "Record<{}, {}>",
                match rtype1.as_ref() {
                    RustType::Simple { id } if generic_types.contains(id) => {
                        return Err(LanguageError::from(
                            RustTypeFormatError::GenericKeyForbiddenInTS(id.clone()),
                        ));
                    }
                    _ => self.format_type(rtype1, generic_types)?,
                },
                self.format_type(rtype2, generic_types)?
            )),
            SpecialRustType::Unit => Ok("undefined".into()),
            SpecialRustType::String => Ok("string".into()),
            SpecialRustType::Char => Ok("string".into()),
            SpecialRustType::I8
            | SpecialRustType::U8
            | SpecialRustType::I16
            | SpecialRustType::U16
            | SpecialRustType::I32
            | SpecialRustType::U32
            | SpecialRustType::I54
            | SpecialRustType::U53
            | SpecialRustType::F32
            | SpecialRustType::F64 => Ok("number".into()),
            SpecialRustType::Bool => Ok("boolean".into()),
            SpecialRustType::U64
            | SpecialRustType::I64
            | SpecialRustType::ISize
            | SpecialRustType::USize => {
                if self.config.use_bigint {
                    Ok("bigint".into())
                } else {
                    Err(TypescriptError::No64BitIntegerType.into())
                }
            }
        }
    }

    fn write_comment(
        &mut self,
        w: &mut impl Write,
        comment: &Comment<'_>,
    ) -> LangResult<(), Self::Error> {
        if !comment.is_empty() {
            let tab_indent = match comment.get_location() {
                CommentLocation::FileHeader => String::new(),
                CommentLocation::Type => String::new(),
                CommentLocation::Field => "\t".repeat(1),
            };
            let comment: String = match comment {
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
                Comment::MultilineOwned { comment, .. } => {
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

            writeln!(w, "{}", comment)?;
        }
        Ok(())
    }

    fn write_type_alias(&mut self, w: &mut impl Write, ty: &RustTypeAlias) -> TypescriptResult<()> {
        self.write_comment(w, &ty.comments)?;

        let r#type = self.format_type(&ty.r#type, ty.generic_types.as_slice())?;

        writeln!(
            w,
            "export type {}{} = {}{};\n",
            ty.id.renamed,
            (!ty.generic_types.is_empty())
                .then(|| format!("<{}>", ty.generic_types.join(", ")))
                .unwrap_or_default(),
            r#type,
            ty.r#type
                .is_optional()
                .then(|| " | undefined")
                .unwrap_or_default(),
        )?;

        Ok(())
    }

    fn write_struct(&mut self, w: &mut impl Write, rs: &RustStruct) -> TypescriptResult<()> {
        self.write_comment(w, &rs.comments)?;
        writeln!(
            w,
            "export interface {}{} {{",
            rs.id.renamed,
            (!rs.generic_types.is_empty())
                .then(|| format!("<{}>", rs.generic_types.join(", ")))
                .unwrap_or_default()
        )?;

        rs.fields
            .iter()
            .try_for_each(|f| self.write_field(w, f, rs.generic_types.as_slice()))?;

        writeln!(w, "}}\n")?;
        Ok(())
    }

    fn write_enum(&mut self, w: &mut impl Write, e: &RustEnum) -> TypescriptResult<()> {
        self.write_comment(w, &e.shared().comments)?;

        let generic_parameters = (!e.shared().generic_types.is_empty())
            .then(|| format!("<{}>", e.shared().generic_types.join(", ")))
            .unwrap_or_default();

        match e {
            RustEnum::Unit(shared) => {
                write!(
                    w,
                    "export enum {}{} {{",
                    shared.id.renamed, generic_parameters
                )?;

                self.write_enum_variants(w, e)?;

                writeln!(w, "\n}}\n")?;
            }
            RustEnum::Algebraic { shared, .. } => {
                if EnumWriteMethod::ManyTypes == self.config.enum_write_method {
                    self.write_comment(w, &shared.comments)?;
                    for variant in &shared.variants {
                        match variant {
                            RustEnumVariant::AnonymousStruct {
                                shared: variant_shared,
                                fields,
                            } => {
                                let variant_name =
                                    format!("{}{}", shared.id.original, variant_shared.id.original);

                                self.write_comment(w, &shared.comments)?;
                                writeln!(
                                    w,
                                    "export interface {}{} {{",
                                    variant_name,
                                    (!shared.generic_types.is_empty())
                                        .then(|| format!("<{}>", shared.generic_types.join(", ")))
                                        .unwrap_or_default()
                                )?;

                                fields.iter().try_for_each(|f| {
                                    self.write_field(w, f, shared.generic_types.as_slice())
                                })?;
                                writeln!(w, "}}\n")?;
                            }
                            _ => {}
                        }
                    }
                }
                write!(
                    w,
                    "export type {}{} = ",
                    shared.id.renamed, generic_parameters
                )?;

                self.write_enum_variants(w, e)?;

                write!(w, ";")?;
                writeln!(w)?;
                writeln!(w)?;
            }
        }
        Ok(())
    }
}

impl TypeScript {
    fn write_enum_variants(&mut self, w: &mut impl Write, e: &RustEnum) -> TypescriptResult<()> {
        match e {
            // Write all the unit variants out (there can only be unit variants in
            // this case)
            RustEnum::Unit(shared) => {
                for variant in &shared.variants {
                    match variant {
                        RustEnumVariant::Unit(shared) => {
                            writeln!(w)?;
                            self.write_comment(w, &shared.comments)?;
                            write!(w, "\t{} = {:?},", shared.id.original, &shared.id.renamed)?;
                        }
                        _ => unreachable!(),
                    }
                }
            }

            // Write all the algebraic variants out (all three variant types are possible
            // here)
            RustEnum::Algebraic {
                tag_key,
                content_key,
                shared,
            } => {
                for v in &shared.variants {
                    {
                        writeln!(w)?;
                        self.write_comment(w, &v.shared().comments)?;
                        match v {
                            RustEnumVariant::Unit(shared) => {
                                write!(
                                    w,
                                    "\t| {{ {}: {:?}, {}?: undefined }}",
                                    tag_key, shared.id.renamed, content_key
                                )?;
                            }
                            RustEnumVariant::Tuple { ty, shared } => {
                                let r#type =
                                    self.format_type(ty, e.shared().generic_types.as_slice())?;
                                write!(
                                    w,
                                    "\t| {{ {}: {:?}, {}{}: {} }}",
                                    tag_key,
                                    shared.id.renamed,
                                    content_key,
                                    ty.is_optional().then(|| "?").unwrap_or_default(),
                                    r#type
                                )?;
                            }
                            RustEnumVariant::AnonymousStruct {
                                fields,
                                shared: variant_shared,
                            } => {
                                write!(
                                    w,
                                    "\t| {{ {}: {:?}, {}: ",
                                    tag_key, variant_shared.id.renamed, content_key
                                )?;

                                if EnumWriteMethod::ManyTypes == self.config.enum_write_method {
                                    write!(
                                        w,
                                        "{}{}",
                                        shared.id.original, variant_shared.id.original,
                                    )?;
                                } else {
                                    write!(w, "{{")?;
                                    fields.iter().try_for_each(|f| {
                                        self.write_field(w, f, e.shared().generic_types.as_slice())
                                    })?;

                                    write!(w, "}}")?;
                                }
                                write!(w, "}}\n")?;
                            }
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    fn write_field(
        &mut self,
        w: &mut impl Write,
        field: &RustField,
        generic_types: &[String],
    ) -> TypescriptResult<()> {
        if field.comments.is_empty() {
            if let Some(comments) = self.config.type_mappings.get_comments(&field.ty.id()) {
                let comments = comments.clone();
                self.write_comment(w, &comments)?;
            }
        } else {
            self.write_comment(w, &field.comments)?;
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
