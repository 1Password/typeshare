use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn build_typeshare_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let module = syn::parse_macro_input!(item as syn::ItemMod);
    let module_name = module.ident.clone();

    let result = quote! {
        #module

        #[doc(hidden)]
        #[cfg(not(target_arch = "wasm32"))]
        pub mod native_typeshare_module {
            use super::#module_name as lang_impl_module;
            use typeshare_module::ffi_interop::ffi_v1;
            #[no_mangle]
            pub extern "C" fn init_logger(log: ffi_v1::FFILanguageLoggerConfig) {
                typeshare_module::init_logger("typescript", log.into());
            }
            #[no_mangle]
            pub extern "C" fn language_config(
            ) -> ffi_v1::FFIArray<ffi_v1::FFIArgumentRef> {
                let vec =
                <lang_impl_module::TypeConfig as typeshare_module::argument::LanguageArguments>::get_arguments();
                vec.try_into().unwrap()
            }
            #[no_mangle]
            pub extern "C" fn default_config() -> ffi_v1::FFIString {
                if let Ok(value) = ffi_v1::FFIString::try_from(lang_impl_module::get_default_config()){
                    return value;
                }else{
                    typeshare_module::log::warn!("Default Config could not be converted to FFIString");
                    return ffi_v1::FFIString::default();
                }
            }

            #[no_mangle]
            pub extern "C" fn language_module() -> ffi_v1::FFILanguageModule {
                ffi_v1::FFILanguageModule::new(
                    lang_impl_module::LANGUAGE_NAME.to_string(),
                    env!("CARGO_PKG_VERSION").to_string(),
                    vec![],
                    vec![],
                    None,
                )
            }
            #[no_mangle]
        pub extern "C" fn build_types(
            mut config: ffi_v1::FFIMap,
            file: ffi_v1::FFIString,
            parsed_data: ffi_v1::raw_parsed_data::RawParsedData,
        ) -> u32 {
                let file: Result<String, _> = unsafe { file.try_into() };
            let Ok(file) = file else {
                return 1;
            };
            let file = std::path::PathBuf::from(file);

            let parsed_data = parsed_data.into_parsed_data();
            let config:  Result<lang_impl_module::TypeConfig, ffi_v1::FFIString> = ffi_v1::parse_ffi_map(config);
            // Catch Config Error
            let result = lang_impl_module::build_types(config.unwrap(), parsed_data);
            match result {
                Ok((write, default_file)) => {
                    match write.write_parse_with_default_file(file, &default_file) {
                        Ok(_) => 0,
                        Err(err) => {
                            typeshare_module::log::error!("{:#?}", err);
                            return 4;
                        }
                    }
                }
                Err(err) => {
                    typeshare_module::log::error!("{:#?}", err);
                    return 3;
                }
            }
        }
    }
        };
    result.into()
}
