use std::io::Write;

use typeshare_core::{
    language::{Generate, TypeFormatter},
    parsed_types::ParsedStruct,
};

use crate::{lang_impl::TypescriptResult, TypeScript};

impl Generate<TypeScript> for ParsedStruct {
    fn generate_to(
        &self,
        language: &mut TypeScript,
        write: &mut impl Write,
    ) -> TypescriptResult<()> {
        self.shared().comments.generate_to(language, write)?;
        match self {
            ParsedStruct::TraditionalStruct { fields, shared } => {
                writeln!(
                    write,
                    "export interface {}{} {{",
                    shared.id.renamed,
                    (!shared.generic_types.is_empty())
                        .then(|| format!("<{}>", shared.generic_types.join(", ")))
                        .unwrap_or_default()
                )?;

                fields.iter().try_for_each(|f| {
                    language.write_field(write, f, shared.generic_types.as_slice())
                })?;

                writeln!(write, "}}\n")?;
            }
            ParsedStruct::SerializedAs { value_type, shared } => {
                let formatted_type =
                    language.format_type(value_type, shared.generic_types.as_slice())?;

                writeln!(
                    write,
                    "export type {}{} = {}{};\n",
                    shared.id.renamed,
                    (!shared.generic_types.is_empty())
                        .then(|| format!("<{}>", shared.generic_types.join(", ")))
                        .unwrap_or_default(),
                    formatted_type,
                    value_type
                        .is_optional()
                        .then(|| " | undefined")
                        .unwrap_or_default(),
                )?;
            }
        }
        Ok(())
    }
}
