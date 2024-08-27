# Target OS

The `--target-os` argument is an optional command line argument that allows you to specify a list of comma separated `target_os` values. In your
Rust source code you can use the [`target_os`](https://doc.rust-lang.org/reference/conditional-compilation.html#target_os) attribute
to restrict a type, variant or fields.

If you do not use `--target-os` then typeshare will generate all types, variants and fields that are typeshared.

## Example
```
./typeshare ./my_rust_project \
  --lang=typescript \
  --output-file=my_typescript_definitions.ts \
  --target-os=linux,macos
```

The example above is stating any types, variants, fields that are typeshared and are not applicable for `linux` or `macos` will be omitted from
typeshare type generation.

## Supported `target_os` definitions.

### Simple standalone.

```rust
#[cfg(target_os = "android")]
pub struct MyType;
```

This type will only be generated if `--target-os` has `android` in the list of target_os values.

### Simple not rule

```rust
#[cfg(not(target_os = "android"))]
pub struct MyType;
```
This type will only be generated if `--target-os` does not include `android` in the list of target_os values.

### Multiple not any rule

```rust
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub struct MyType;
```

This type will only be generated if `--target-os` does not include `android` or `ios` in the list of target_os values.

The following example will allow `MyType` to be typeshared.
```
./typeshare ./my_rust_project \
  --lang=typescript \
  --output-file=my_typescript_definitions.ts \
  --target-os=linux,macos
```

The following example will not allow `MyType` to be typeshared.
```
./typeshare ./my_rust_project \
  --lang=typescript \
  --output-file=my_typescript_definitions.ts \
  --target-os=android,macos
```

## Combined with features or other cfg attributes

Typehsare will not take into consideration any other `cfg` attributes other than `target_os` when generating types.

For example:

```rust
#[cfg(any(target_os = "android", feature = "android-test")]
pub struct MyType;
```

```rust
#[cfg(all(target_os = "android", feature = "android-test")]
pub struct MyType;
```

```
./typeshare ./my_rust_project \
  --lang=typescript \
  --output-file=my_typescript_definitions.ts \
  --target-os=android
```

In both examples above, `MyType` will be typeshared.
