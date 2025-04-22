#[doc(hidden)]
pub mod ඞ {
    pub use ::anyhow;
    pub use ::typeshare_engine as engine;
}

#[doc(hidden)]
#[macro_export]
macro_rules! type_lifetime_helper {
    ($lt:lifetime, $Language:ident) => {$Language};
    ($lt:lifetime, $Language:ident < $lt2:lifetime >) => {$Language<$lt>};
}

#[macro_export]
macro_rules! typeshare_binary {
    ($($Language:ident $(< $config:lifetime >)?),+ $(,)?) => {
        fn main() -> $crate::ඞ::anyhow::Result<()> {
            struct Local;

            impl $crate::ඞ::engine::driver::LanguageHelper for Local {
                type LanguageSet<'config> = ($(
                    $crate::type_lifetime_helper! ('config, $Language $(<$config>)?),
                )+);
            }

            $crate::ඞ::engine::driver::main_body::<Local>()
        }
    };
}
