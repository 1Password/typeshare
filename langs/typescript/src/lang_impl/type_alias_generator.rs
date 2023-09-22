use std::io::Write;

use typeshare_core::{
    language::{Generate, TypeFormatter},
    parsed_types::ParsedTypeAlias,
};

use crate::{lang_impl::TypescriptResult, TypeScript};

impl Generate<TypeScript> for ParsedTypeAlias {
    fn generate_to(
        &self,
        language: &mut TypeScript,
        write: &mut impl Write,
    ) -> TypescriptResult<()> {
        self.comments.generate_to(language, write)?;

        let r#type = language.format_type(&self.value_type, self.generic_types.as_slice())?;

        writeln!(
            write,
            "export type {}{} = {}{};\n",
            self.id.renamed,
            (!self.generic_types.is_empty())
                .then(|| format!("<{}>", self.generic_types.join(", ")))
                .unwrap_or_default(),
            r#type,
            self.value_type
                .is_optional()
                .then(|| " | undefined")
                .unwrap_or_default(),
        )?;

        Ok(())
    }
}
