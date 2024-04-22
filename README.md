# Typeshare

| Crate                | Status                                                                                                                                                                                                                                                                                                                                                                                                                                      |
|----------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| typeshare            | [![crates.io version](https://img.shields.io/crates/v/typeshare.svg)](https://crates.io/crates/typeshare) [![crate documentation](https://docs.rs/typeshare/badge.svg)](https://docs.rs/typeshare) ![MSRV](https://img.shields.io/badge/rustc-stable-blue.svg) [![crates.io downloads](https://img.shields.io/crates/d/typeshare.svg)](https://crates.io/crates/typeshare)                                                                   |
| typeshare-core       | [![crates.io version](https://img.shields.io/crates/v/typeshare-core.svg)](https://crates.io/crates/typeshare-core) [![crate documentation](https://docs.rs/typeshare-core/badge.svg)](https://docs.rs/typeshare-core) ![MSRV](https://img.shields.io/badge/rustc-stable-blue.svg) [![crates.io downloads](https://img.shields.io/crates/d/typeshare-core.svg)](https://crates.io/crates/typeshare-core)                                     |
| typeshare-annotation | [![crates.io version](https://img.shields.io/crates/v/typeshare-annotation.svg)](https://crates.io/crates/typeshare-annotation) [![crate documentation](https://docs.rs/typeshare-annotation/badge.svg)](https://docs.rs/typeshare-annotation) ![MSRV](https://img.shields.io/badge/rustc-stable-blue.svg) [![crates.io downloads](https://img.shields.io/crates/d/typeshare-annotation.svg)](https://crates.io/crates/typeshare-annotation) |
| typeshare-cli        | [![crates.io version](https://img.shields.io/crates/v/typeshare-cli.svg)](https://crates.io/crates/typeshare-cli) ![MSRV](https://img.shields.io/badge/rustc-stable-blue.svg) [![crates.io downloads](https://img.shields.io/crates/d/typeshare-cli.svg)](https://crates.io/crates/typeshare-cli)                                                                                                                                            |


_One tool to rule the types,_

_One tool to FFI them,_

_One tool to parse your Rust,_

_And in the darkness, compile them_ 💍


Do you like manually managing types that need to be passed through an FFI layer, so that your code doesn't archaically break at runtime? Be honest, nobody does. Typeshare is here to take that burden away from you! Leveraging the power of the `serde` library, Typeshare is a tool that converts your
Rust types into their equivalent forms in Swift, Go**, Kotlin, Scala and Typescript, keeping
your cross-language codebase in sync. With automatic implementation for serialization and deserialization on both sides of the FFI, Typeshare does all the heavy lifting for you. It can even handle generics and convert effortlessly between standard libraries in different languages!

**A few caveats. See [here](#a-quick-refresher-on-supported-languages) for more details.

## Installation


To install the CLI (Command Line Interface):
```
cargo install typeshare-cli
```

💡Note that the console command will be `typeshare`, not `typeshare-cli`.

In your `Cargo.toml`, under `[dependencies]`:

```toml
typeshare = "1.0.0"
```

## Using Typeshare
We've put together a book that documents (almost) everything you can do.

📚[Read the Typeshare book here!](https://1password.github.io/typeshare)

To generate FFI definitions for a target language, run the `typeshare` command and specify the directory containing your rust code, the language you would like to generate for, and the file to which your generated definitions will be written:
```
typeshare ./my_rust_project --lang=kotlin --output-file=my_kotlin_definitions.kt
typeshare ./my_rust_project --lang=swift --output-file=my_swift_definitions.swift
typeshare ./my_rust_project --lang=scala --output-file=my_scala_definitions.scala
typeshare ./my_rust_project --lang=typescript --output-file=my_typescript_definitions.ts
```

### Annotating Types

Include the `#[typeshare]` attribute with any struct or enum you define to generate definitions for that type in the selected output language.

```rust
// Rust type definitions

#[typeshare]
struct MyStruct {
    my_name: String,
    my_age: u32,
}

#[typeshare]
#[serde(tag = "type", content = "content")]
enum MyEnum {
    MyVariant(bool),
    MyOtherVariant,
    MyNumber(u32),
}
```
```typescript
// Generated Typescript definitions

export interface MyStruct {
    my_name: string;
    my_age: number;
}

export type MyEnum =
    | { type: "MyVariant", content: boolean }
    | { type: "MyOtherVariant", content: undefined }
    | { type: "MyNumber", content: number };
```

## Getting Help

Are you getting weird deserialization issues? Did our procedural macro throw a confusing error at you? Are you trying to contribute and our existing codebase is confusing? (probably true) Did you have another problem not enumerated in this reductive list? Please open an issue in this repository and the 1Password team would be happy to help! That's what we're here for!

## A Quick Refresher on Supported Languages

- Kotlin
- Scala
- Swift
- Typescript
- Go**

If there is a language that you want Typeshare to generate definitions for, you can either:
1. Open an issue in this repository requesting your language of choice.
2. Implement support for that language and open a PR with your implementation. We would be eternally grateful! 🙏

** Right now, Go support is experimental. Enable the `go` feature when installing typeshare-cli if you want to use it.

## Credits

Made with ❤️ and ☕ by the [1Password](https://1password.com/) team.

### Get a free 1Password account for your open source project

Does your team need a secure way to manage passwords and other credentials for your open source project? Head on over to our [other repository](https://github.com/1Password/1password-teams-open-source) to get a 1Password Teams account on us:

✨[1Password for Open Source Projects](https://github.com/1Password/1password-teams-open-source)

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
