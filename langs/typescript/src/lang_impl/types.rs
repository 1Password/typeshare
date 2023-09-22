use joinery::JoinableIterator;
use typeshare_core::{
    language::TypeFormatter,
    parsed_types::{Number, SpecialType, Type},
};

use crate::{
    lang_impl::{TypescriptError, TypescriptResult},
    TypeScript,
};

impl TypeFormatter for TypeScript {
    fn format_special_type(
        &mut self,
        special_ty: &SpecialType,
        generic_types: &[String],
    ) -> TypescriptResult<String> {
        if let Some(type_override) = self.config.type_mappings.get(special_ty.id()) {
            return Ok(type_override.to_string());
        }
        match special_ty {
            SpecialType::Vec(rtype) => Ok(format!("{}[]", self.format_type(rtype, generic_types)?)),
            SpecialType::Array(rtype, len) => {
                let formatted_type = self.format_type(rtype, generic_types)?;
                Ok(format!(
                    "[{}]",
                    std::iter::repeat(&formatted_type)
                        .take(*len)
                        .join_with(", ")
                ))
            }
            SpecialType::Slice(rtype) => {
                Ok(format!("{}[]", self.format_type(rtype, generic_types)?))
            }
            // We add optionality above the type formatting level
            SpecialType::Option(rtype) => self.format_type(rtype, generic_types),
            SpecialType::Map(rtype1, rtype2) => Ok(format!(
                "Record<{}, {}>",
                match rtype1.as_ref() {
                    Type::Simple { id } if generic_types.contains(id) => {
                        return Err(TypescriptError::GenericKeyForbiddenInTS(id.clone()).into());
                    }
                    _ => self.format_type(rtype1, generic_types)?,
                },
                self.format_type(rtype2, generic_types)?
            )),
            SpecialType::Unit => Ok("undefined".into()),
            SpecialType::String => Ok("string".into()),
            SpecialType::Char => Ok("string".into()),
            SpecialType::Number(number) => match number {
                Number::I8
                | Number::U8
                | Number::I16
                | Number::U16
                | Number::I32
                | Number::U32
                | Number::I54
                | Number::U53
                | Number::F32
                | Number::F64 => Ok("number".into()),
                Number::U64 | Number::I64 | Number::ISize | Number::USize => {
                    if self.config.use_bigint {
                        Ok("bigint".into())
                    } else {
                        Err(TypescriptError::No64BitIntegerType.into())
                    }
                }
            },
            SpecialType::Bool => Ok("boolean".into()),
        }
    }
}
