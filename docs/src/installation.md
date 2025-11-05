# Installation

Identify which typeshare implements the language(s) you'd like to use, and install it with `cargo install`. The official typeshare binary supports Kotlin, Swift, and Typescript:

```
cargo install typeshare2-cli
```

There are also third party crates implementing other languages using typeshare, such as:

- Java: `cargo install typeshare-java`

Once you have the CLI installed, you then need to annotate the rust types that you want to generate FFI definitions for. In order to be able to use the `#[typeshare]` annotation, you will need to add `typeshare` as a dependency to your project's `Cargo.toml`.

```toml
# Cargo.toml

[dependencies]
typeshare = "1.0.0" # Use whichever version is the most recent
```
