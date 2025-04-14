#[doc(hidden)]
pub mod reexport {
    pub use ::anyhow;
    pub use ::typeshare_engine as engine;
}

#[macro_export]
macro_rules! typeshare_binary {
    ($($Language:ident),+ $(,)?) => {
        fn main() -> $crate::reexport::anyhow::Result<()> {
            $crate::reexport::engine::driver::main_body::<($(
                $Language,
            )*)>()
        }
    };
}
