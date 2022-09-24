# Installation

The easiest way to install the Typeshare CLI is with `cargo`. Just run the following command:
```
cargo install typeshare-cli
```

Once you have the CLI installed, you then need to annotate the rust types that you want to generate FFI definitions for. In order to be able to use the `#[typeshare]` annotation, you will need to add `typeshare` as a dependency to your project's `Cargo.toml`.

```toml
# Cargo.toml

[dependencies]
typeshare = "0.1.0" # Use whichever version is the most recent
```
