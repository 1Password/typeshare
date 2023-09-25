use crate::argument::{ArgumentRef, ArgumentType, CLIArgument};
use crate::ffi_interop::ffi_v1::ffi_string::FFIString;
use crate::ffi_interop::ffi_v1::ffi_value::OptionalFFIValue;
use crate::ffi_interop::ffi_v1::{FFIArray, FFIType, OptionalFFIString};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[repr(C)]
#[serde(tag = "type", content = "value")]
pub enum FFIArgumentType {
    String,
    Enum(FFIArray<FFIString>),
    Bool,
    Number,
    SignedNumber,
    Float,
    FilePath,
    Array(Box<FFIArgumentType>),
    StringMap(Box<FFIArgumentType>),
    TypeMap,
}
impl FFIType for FFIArgumentType {
    type SafeType = ArgumentType;
}
impl From<ArgumentType> for FFIArgumentType {
    fn from(value: ArgumentType) -> Self {
        match value {
            ArgumentType::String => Self::String,
            ArgumentType::Enum(variants) => {
                let variants: FFIArray<FFIString> = variants.try_into().unwrap();
                Self::Enum(variants)
            }
            ArgumentType::Bool => Self::Bool,
            ArgumentType::Number => Self::Number,
            ArgumentType::SignedNumber => Self::SignedNumber,
            ArgumentType::Float => Self::Float,
            ArgumentType::FilePath => Self::FilePath,
            ArgumentType::Array(inner) => Self::Array(Box::new((*inner).into())),
            ArgumentType::StringMap(inner) => Self::StringMap(Box::new((*inner).into())),
            ArgumentType::TypeMap => Self::TypeMap,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize)]
#[repr(C)]
pub struct FFIArgumentRef {
    pub name: FFIString,
    pub argument_type: FFIArgumentType,
    pub default_value: OptionalFFIValue,
    pub required: bool,
    pub help: OptionalFFIString,
    pub cli: FFICLIArgument,
}
impl FFIType for FFIArgumentRef {
    type SafeType = ArgumentRef;
}
impl From<ArgumentRef> for FFIArgumentRef {
    fn from(value: ArgumentRef) -> Self {
        let ArgumentRef {
            name,
            argument_type,
            required,
            default_value,
            help,
            cli,
        } = value;
        let name = name.try_into().unwrap();
        let help = help.try_into().unwrap();
        let cli = cli.into();
        let default_value = default_value.try_into().unwrap();
        Self {
            name,
            argument_type: argument_type.into(),
            default_value,
            required,
            help,
            cli,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize)]
#[repr(C)]
#[serde(tag = "type", content = "value")]
pub enum FFICLIArgument {
    CLI {
        long: OptionalFFIString,
        short: OptionalFFIString,
    },
    NoCLI,
}
impl FFIType for FFICLIArgument {
    type SafeType = CLIArgument;
}
impl From<CLIArgument> for FFICLIArgument {
    fn from(value: CLIArgument) -> Self {
        match value {
            CLIArgument::CLI { long, short } => {
                let long = long.try_into().unwrap();
                let short = short.try_into().unwrap();
                Self::CLI { long, short }
            }
            CLIArgument::NoCLI => Self::NoCLI,
        }
    }
}
impl Default for FFICLIArgument {
    fn default() -> Self {
        Self::NoCLI
    }
}
