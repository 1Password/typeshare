use std::io::Write;

use crate::config::EnumWriteMethod;
use crate::lang_impl::TypescriptResult;
use crate::TypeScript;
use typeshare_core::{
    language::{Generate, TypeFormatter},
    parsed_types::{AlgebraicEnum, AnonymousStructVariant, EnumVariant, ParsedEnum, TupleVariant},
};

fn write_enum_variants(
    language: &mut TypeScript,
    w: &mut impl Write,
    e: &ParsedEnum,
) -> TypescriptResult<()> {
    match e {
        // Write all the unit variants out (there can only be unit variants in
        // this case)
        ParsedEnum::Unit(shared) => {
            for variant in &shared.variants {
                match variant {
                    EnumVariant::Unit(shared) => {
                        writeln!(w)?;
                        shared.comments.generate_to(language, w)?;
                        write!(w, "\t{} = {:?},", shared.id.original, &shared.id.renamed)?;
                    }
                    _ => unreachable!(),
                }
            }
        }

        // Write all the algebraic variants out (all three variant types are possible
        // here)
        ParsedEnum::Algebraic(algebraic) => {
            algebraic.generate_to(language, w)?;
        }
        ParsedEnum::SerializedAs { .. } => {
            todo!("SerializedAs enums are not supported yet")
        }
    }
    return Ok(());
}
impl Generate<TypeScript> for AlgebraicEnum {
    fn generate_to(
        &self,
        language: &mut TypeScript,
        write: &mut impl Write,
    ) -> TypescriptResult<()> {
        let Self {
            tag_key,
            content_key,
            shared,
            ..
        } = self;
        for v in &shared.variants {
            writeln!(write)?;
            shared.comments.generate_to(language, write)?;
            match v {
                EnumVariant::Unit(shared) => {
                    write!(
                        write,
                        "\t| {{ {}: {:?}, {}?: undefined }}",
                        tag_key, shared.id.renamed, content_key
                    )?;
                }
                EnumVariant::Tuple(TupleVariant {
                    ty,
                    shared: shared_variant,
                }) => {
                    let r#type = language.format_type(ty, shared.generic_types.as_slice())?;
                    write!(
                        write,
                        "\t| {{ {}: {:?}, {}{}: {} }}",
                        tag_key,
                        shared_variant.id.renamed,
                        content_key,
                        ty.is_optional().then(|| "?").unwrap_or_default(),
                        r#type
                    )?;
                }
                EnumVariant::AnonymousStruct(AnonymousStructVariant {
                    shared: variant_shared,
                    fields,
                }) => {
                    write!(
                        write,
                        "\t| {{ {}: {:?}, {}: ",
                        tag_key, variant_shared.id.renamed, content_key
                    )?;

                    if EnumWriteMethod::ManyTypes == language.config.enum_write_method {
                        write!(
                            write,
                            "{}{}",
                            shared.id.original, variant_shared.id.original,
                        )?;
                    } else {
                        write!(write, "{{")?;
                        fields.iter().try_for_each(|f| {
                            language.write_field(write, f, shared.generic_types.as_slice())
                        })?;

                        write!(write, "}}")?;
                    }
                    write!(write, "}}\n")?;
                }
            }
        }
        Ok(())
    }
}
impl Generate<TypeScript> for ParsedEnum {
    fn generate_to(
        &self,
        language: &mut TypeScript,
        write: &mut impl Write,
    ) -> TypescriptResult<()> {
        self.shared().comments.generate_to(language, write)?;

        let generic_parameters = (!self.shared().generic_types.is_empty())
            .then(|| format!("<{}>", self.shared().generic_types.join(", ")))
            .unwrap_or_default();

        match self {
            ParsedEnum::Unit(shared) => {
                write!(
                    write,
                    "export enum {}{} {{",
                    shared.id.renamed, generic_parameters
                )?;

                write_enum_variants(language, write, self)?;

                writeln!(write, "\n}}\n")?;
            }
            ParsedEnum::Algebraic(AlgebraicEnum { shared, .. }) => {
                if EnumWriteMethod::ManyTypes == language.config.enum_write_method {
                    shared.comments.generate_to(language, write)?;
                    for variant in &shared.variants {
                        match variant {
                            EnumVariant::AnonymousStruct(AnonymousStructVariant {
                                shared: variant_shared,
                                fields,
                            }) => {
                                let variant_name =
                                    format!("{}{}", shared.id.original, variant_shared.id.original);

                                shared.comments.generate_to(language, write)?;
                                writeln!(
                                    write,
                                    "export interface {}{} {{",
                                    variant_name,
                                    (!shared.generic_types.is_empty())
                                        .then(|| format!("<{}>", shared.generic_types.join(", ")))
                                        .unwrap_or_default()
                                )?;

                                fields.iter().try_for_each(|f| {
                                    language.write_field(write, f, shared.generic_types.as_slice())
                                })?;
                                writeln!(write, "}}\n")?;
                            }
                            _ => {}
                        }
                    }
                }
                write!(
                    write,
                    "export type {}{} = ",
                    shared.id.renamed, generic_parameters
                )?;

                write_enum_variants(language, write, self)?;

                write!(write, ";")?;
                writeln!(write)?;
                writeln!(write)?;
            }
            ParsedEnum::SerializedAs { .. } => {
                todo!("SerializedAs enums are not supported yet")
            }
        }
        Ok(())
    }
}
