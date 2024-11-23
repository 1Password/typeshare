# typeshare-driver

This crate contains a macro generating all the glue code necessary to create a typeshare binary. The idea is that person wanting a typeshare binary supporting
certain languages would be able to write something as simple as:

```rust
use typeshare_swift::Swift;
use typeshare_typescript::Typescript;
use typeshare_driver::generate_typeshare_cli

generate_typeshare_cli!(Swift, Typescript)
```

This would create a `fn main` that uses these languages, plus `typeshare-engine`, that implements a full typeshare CLI.

This crate is among the last things I want to work on, because making it sufficiently configurable might be tricky.

Theoretically, if (for example) `typeshare-python` wanted to export a full `typeshare-python` binary, it could depend on both `typeshare-model` (for the implementation) and `typeshare-main` for the binary, exposing the latter probably as a cargo feature.
