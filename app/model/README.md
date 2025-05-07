# typeshare-model

This crate is the core dependency for all implementations of specific languages for typeshare. It defines the [`Language`](https://docs.rs/typeshare-model/latest/typeshare_model/trait.Language.html) trait, along with a handful of supporting types, which a language implementation must implement. Check out the official implementations for [Swift](../langs/swift/src/lib.rs), [Kotlin](../langs/kotlin/src/lib.rs), and [Typescript](../langs/typescript/src/lib.rs) for examples.
