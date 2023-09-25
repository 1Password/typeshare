use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIs};
use typeshare_core::value::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIs, Display, AsRefStr)]
#[repr(C)]
#[serde(tag = "type", content = "value")]
pub enum ArgumentType {
    String,
    Enum(Vec<String>),
    Bool,
    Number,
    SignedNumber,
    Float,
    FilePath,
    Array(Box<ArgumentType>),
    StringMap(Box<ArgumentType>),
    TypeMap,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArgumentRef {
    pub name: String,
    pub argument_type: ArgumentType,
    pub required: bool,
    pub default_value: Option<Value>,
    pub help: Option<String>,
    pub cli: CLIArgument,
}
pub trait LanguageArguments {
    fn get_arguments() -> Vec<ArgumentRef>
    where
        Self: Sized;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CLIArgument {
    CLI {
        long: Option<String>,
        short: Option<String>,
    },
    NoCLI,
}

impl Default for CLIArgument {
    fn default() -> Self {
        Self::NoCLI
    }
}
mod ffi_v1 {
    use super::{ArgumentRef, ArgumentType, CLIArgument};
    use crate::ffi_interop::ffi_v1::{FFIArgumentRef, FFIArgumentType, FFICLIArgument};

    impl From<FFICLIArgument> for CLIArgument {
        fn from(value: FFICLIArgument) -> Self {
            match value {
                FFICLIArgument::CLI { long, short } => {
                    let long = long.try_into().unwrap();
                    let short = short.try_into().unwrap();
                    Self::CLI { long, short }
                }
                FFICLIArgument::NoCLI => Self::NoCLI,
            }
        }
    }
    impl From<FFIArgumentRef> for ArgumentRef {
        fn from(value: FFIArgumentRef) -> Self {
            let FFIArgumentRef {
                name,
                argument_type,
                required,
                default_value,
                help,
                cli,
            } = value;
            let name = name.to_string();
            let help = help.into();
            let cli = cli.into();
            let default_value = default_value.into();
            Self {
                name,
                argument_type: argument_type.into(),
                required,
                default_value,
                help,
                cli,
            }
        }
    }
    impl From<FFIArgumentType> for ArgumentType {
        fn from(value: FFIArgumentType) -> Self {
            match value {
                FFIArgumentType::String => Self::String,
                FFIArgumentType::Enum(variants) => {
                    let variants: Vec<String> = variants.try_into().unwrap();
                    Self::Enum(variants)
                }
                FFIArgumentType::Bool => Self::Bool,
                FFIArgumentType::Number => Self::Number,
                FFIArgumentType::SignedNumber => Self::SignedNumber,
                FFIArgumentType::Float => Self::Float,
                FFIArgumentType::FilePath => Self::FilePath,
                FFIArgumentType::Array(inner) => Self::Array(Box::new((*inner).into())),
                FFIArgumentType::StringMap(inner) => Self::StringMap(Box::new((*inner).into())),
                FFIArgumentType::TypeMap => Self::TypeMap,
            }
        }
    }
}
