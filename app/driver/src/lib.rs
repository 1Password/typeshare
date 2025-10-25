/*!
Framework for creating a typeshare binary.

This library provides a macro that creates a `fn main` that implements an
entire typeshare binary, using only the [Language](https://docs.rs/typeshare-model/latest/typeshare_model/trait.Language.html)
implementations that you provide.

The macro is very simple. Supposing that you have implementations of `Language`
called `Kotlin` and `Swift`, call the macro like this:

```ignore
use std::marker::PhantomData;
use typeshare_driver::typeshare_binary;

struct Kotlin {}

// impl<'config> Language<'config> for Kotlin { ... }

struct Swift<'c> {
    config: PhantomData<&'config str>;
}

// impl<'config> Language<'config> for Swift<'config> { ... }

typeshare_binary! { Kotlin, Swift<'config> }
```

This creates an `fn main` that uses the functionality in [typeshare-engine][typeshare_engine]
to create a complete, working typeshare binary. That binary will include a
complete command-line interface, populated with global options like `--config`
and `--lang`, as well as language-specific options like `--kotlin-package`;
these language-specific flags are determined automatically based on the
[`Config`](https://docs.rs/typeshare-model/latest/typeshare_model/trait.Language.html#associatedtype.Config)
type provided by each `Language` implementation. Use `--help` for a complete
description of all CLI options.

See the [`typeshare_model::Language`](https://docs.rs/typeshare-model/latest/typeshare_model/trait.Language.html)
docs for details on how to create a specific language implementation.

See the [typeshare-engine][typeshare_engine] docs if you want to use typeshare
as a library instead of a binary program; it contains all of the actual logic
for *running* typeshare. typeshare-driver just bootstraps the functionality
in the engine into a working `main`.
*/

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

/**
Macro that creates an `fn main` with a complete typeshare program, based on
the `Language` implementations provided. See the [crate docs][crate] for
details and an example.
*/
#[macro_export]
macro_rules! typeshare_binary {
    ($($Language:ident $(< $config:lifetime >)?),+ $(,)?) => {
        fn main() {
            struct Local;

            impl $crate::ඞ::engine::driver::LanguageHelper for Local {
                type LanguageSet<'config> = ($(
                    $crate::type_lifetime_helper! ('config, $Language $(<$config>)?),
                )+);
            }

            if let Err(err) = $crate::ඞ::engine::driver::main_body::<Local>(
                $crate::ඞ::engine::args::PersonalizeClap::new()
                    .name(env!("CARGO_PKG_NAME"))
                    .version(env!("CARGO_PKG_VERSION")),
            ) {
                log::error!("Typeshare failed: {err}");
                log::error!("{}", err.root_cause());
            }
        }
    };
}
