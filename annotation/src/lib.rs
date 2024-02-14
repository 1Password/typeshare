//! Defines the `#[typeshare]` attribute.

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse, Attribute, Data, DeriveInput, Fields};

/// Marks a type as a type shared across the FFI boundary using typeshare.
///
/// The `typeshare` program will process all the types with this attribute. It will ignore all types
/// that do not have this attribute.
///
/// # Example
///
/// ```ignore
///  use typeshare::typeshare;
///
///  #[typeshare]
///  pub struct Person {
///      name: String,
///      email: String,
///  }
/// ```
///
/// If the file above is `src/person.rs`, then `typeshare --lang=typescript src/person.rs` would
/// generate the typescript bindings for the `Person` type.
///
/// # Ignoring enum variants
/// If you don't want to typeshare a certain enum variant, just put a `#[typeshare(skip)]` attribute on
/// that variant.
/// ```ignore
/// use typeshare::typeshare;
///
/// #[typeshare]
/// pub enum SomeEnum {
///     A,
///     #[typeshare(skip)]
///     B, // <- this variant will not be included in the typeshared file(s)
///     C,
/// }
/// ```
#[proc_macro_attribute]
pub fn typeshare(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(mut item) = parse::<DeriveInput>(item.clone()) {
        // We need to remove the #[typeshare] attribute from all data members so the compiler doesn't throw an error.
        strip_configuration_attribute(&mut item);
        TokenStream::from(item.to_token_stream())
    } else {
        item
    }
}

fn strip_configuration_attribute(item: &mut DeriveInput) {
    fn remove_configuration_from_attributes(attributes: &mut Vec<Attribute>) {
        const CONFIG_ATTRIBUTE_NAME: &str = "typeshare";

        attributes.retain(|x| x.path().to_token_stream().to_string() != CONFIG_ATTRIBUTE_NAME);
    }

    fn remove_configuration_from_fields(fields: &mut Fields) {
        for field in fields.iter_mut() {
            remove_configuration_from_attributes(&mut field.attrs);
        }
    }

    match item.data {
        Data::Enum(ref mut data_enum) => {
            for variant in data_enum.variants.iter_mut() {
                remove_configuration_from_attributes(&mut variant.attrs);
                remove_configuration_from_fields(&mut variant.fields);
            }
        }
        Data::Struct(ref mut data_struct) => {
            remove_configuration_from_fields(&mut data_struct.fields);
        }
        Data::Union(ref mut data_union) => {
            for field in data_union.fields.named.iter_mut() {
                remove_configuration_from_attributes(&mut field.attrs);
            }
        }
    };
}
