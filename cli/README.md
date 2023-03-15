# typeshare

CLI built on top of `typeshare-core`. Generate code in different languages from Rust type definitions for FFI interop.

## Usage

```
cargo install typeshare
typeshare --lang=typescript some/file.rs
typeshare --lang=swift some/file.rs
typeshare --lang=kotlin --java-package=com.some.package.name some/file.rs
typeshare --lang=scala --scala-package=com.some.package.name some/file.rs
```

## Generating FFI bindings

Include the typeshare annotation to generate a FFI binding for that function. Available languages are `kotlin` and `swift`.

```rust
#[typeshare(kotlin = "generateTotp", swift = "generate_totp")]
pub fn generate_totp(request: TotpGeneratorRequest) -> Result<TotpGeneratorResponse>`
```

Until the build system is changed to generate bindings during a build, it must be done manually and committed into the repo. FFI bindings are generated into the `ffi/src/generated.rs` file.

To generate all FFI bindings run `make ffi`. When adding or removing FFI bindings, don't forget to include the changes in the `Makefile`.
