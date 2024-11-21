# Version 1.13.2
- Fix binary name in --help --version so typeshare is the name and not typeshare-cli: [#214](https://github.com/1Password/typeshare/pull/214)

# Version 1.13.1
- Fix duplicate root added to walker: [#209](https://github.com/1Password/typeshare/pull/209)
- Only assert if go package is present if generating go types: [#211](https://github.com/1Password/typeshare/pull/211)
- Update shell completions for new generate function: [#212](https://github.com/1Password/typeshare/pull/212)

# Version 1.13.0
- Update how logging is initialized: [#206](https://github.com/1Password/typeshare/pull/206)
- Don't recreate `Codable.swift` when the contents have not changed [#205](https://github.com/1Password/typeshare/pull/205)
- Fix target_os parsing when no --target-os is provided [#204](https://github.com/1Password/typeshare/pull/204)

# Version 1.12.0

- Optional slices in Go no longer trigger a pointer redirection.
- Upgrade to clap 4. This let us remove the dependency on the now unmaintained atty crate.
- wasmbind is now an optional feature

# Version 1.11.0

This release promotes 1.10.0-beta.x to stable, and several new features.

## Since 1.10.0-beta.7

- Multiple `--target-os` is now allowed, and `#[cfg(not(target_os...))]` is now parsed: [#187](https://github.com/1Password/typeshare/pull/187)
- Console output is now handled by flexi_logger: #[187](https://github.com/1Password/typeshare/pull/187)
- Variant types are now explicitly formatted in Go: [#189](https://github.com/1Password/typeshare/pull/189)

## Summary of 1.10.0-beta.x

See the full changelog for more details: https://github.com/1Password/typeshare/blob/main/CHANGELOG.md

- Output can now be split into multiple generated files
- Source is now walked in parallel, increasing speed
- Generic type constraints can now be defined for Swift
- Kotlin's Inline value classes are now supported
- You can now specify that a struct should be "redacted"
  - The effects are language specific. For Kotlin, `toString` is overridden.

# Version 1.10.0-beta

## 1.10.0-beta.7

- Added support for [inline value classes](https://kotlinlang.org/docs/inline-classes.html) in Kotlin - [#182](https://github.com/1Password/typeshare/pull/182)
- Added the ability to specify that a struct should have information redacted - [#170](https://github.com/1Password/typeshare/pull/170)
  - What this means is language-specific. In Kotlin, the toString method is overridden
  - This was actually added in 1.10.0-beta.4 but went unannounced.
- Made the output deterministic (this broke in 1.10.0-beta.6) - [#185](https://github.com/1Password/typeshare/pull/185)
- Algebraic Enum Variant names are now capitalized appropriately - [#183](https://github.com/1Password/typeshare/pull/183)

## 1.10.0-beta.6

- Added support for skipping fields/variants via the `target_os` argument [#176](https://github.com/1Password/typeshare/pull/176)

## 1.10.0-beta.5

- Added support for Swift generic constraints via `#[typeshare(swiftGenericConstraints)]` [#174](https://github.com/1Password/typeshare/pull/174)
- Added Swift config option for defining constraints on `CodableVoid` generated type [#174](https://github.com/1Password/typeshare/pull/174)

## 1.10.0-beta.4

Fixed a bug involving `#[typeshare(skip)]` on fields in struct variants of enums.

## 1.10.0-beta.2

Fixed a bug involving type aliases.

## 1.10.0-beta.0

This release brings support for multiple file generation, allowing splitting generated
files when used in large projects. This can dramatically increase compilation speed of
the generated files and increase maintainability.

This is a _pre-release_ version which may have bugs or break compatibility.

- Multiple file output [#166](https://github.com/1Password/typeshare/pull/166)

# Version 1.9.2

This release fixes a Cargo.lock error introduced in 1.9.1.

# Version 1.9.1

This release fixes a bug with Kotlin prefixes introduced in 1.9.0.

- Fix inner class referencing incorrect superclass referencing in Kotlin. [#165](https://github.com/1Password/typeshare/pull/165)

# Version 1.9.0

This release adds support for prefixing Kotlin type names (similarly to Swift) and some minor fixes.

- Added support for prefixing type names in Kotlin. [#159](https://github.com/1Password/typeshare/pull/159)

# Version 1.8.0

This release brings support for various Rust std smart pointers, as well as a CLI flag to opt-into following symbolic links. In addition, typeshare has been updated to use syn 2.0

- Added support for various Rust std smart pointers. [#134](https://github.com/1Password/typeshare/pull/134)
- Added CLI flag to opt-into following symbolic links. [#156](https://github.com/1Password/typeshare/pull/156)
- Migrate to syn version 2.0. [#130](https://github.com/1Password/typeshare/pull/130)

### Community contributors

Thank you to the following community contributors for your work on this release:

- [czocher](https://github.com/czocher)
- [ipetkov](https://github.com/ipetkov)

# Version 1.7.0

This release brings support for more rust primitive types (slices and chars), as well as support for manually overriding the output type in the `#[typeshare]` annotations

- Added support for the Rust slice type, which is treated as a sequence. [#131](https://github.com/1Password/typeshare/pull/131)
- Added support for the Rust char type, which is treated as a string. [#128](https://github.com/1Password/typeshare/pull/128)
- Better error messages when there's an error reading a file. [#117](https://github.com/1Password/typeshare/pull/117)
- It is now possible to manually override the output type for specific fields using the `#[typeshare]` annotation. [#119](https://github.com/1Password/typeshare/pull/119), [#118](https://github.com/1Password/typeshare/pull/118)
- Fixed: in Swift, apply generic constraints to enums, in addition to structs. [#122](https://github.com/1Password/typeshare/pull/122)
- In an effort to ensure we don't accidentally break compatibility with our Minimum Supported Rust Version, we added a `rust-toolchain.toml` to the rust crates, forcing builds and tests to use that version of rust. [#129](https://github.com/1Password/typeshare/pull/129), [#135](https://github.com/1Password/typeshare/pull/135). This change should have no effect on end users.

### Community contributors

Thank you to the following community contributors for your work on this release:

- [czocher](https://github.com/czocher)
- [xhain](https://github.com/xhain)

# Version 1.6.0

This release brings support for more architectures for Nix and configurable generic constraints in Swift

- The Nix flake now supports all default Nix platforms, rather than only `x86_64-linux`. [#113](https://github.com/1Password/typeshare/pull/113)
- `typeshare-core`
  - It is now possible to add your own set of generic constraints to Swift generics. This is done with a field called `default_generic_constraints` under `[swift]` in `typeshare.toml`. [#95](https://github.com/1Password/typeshare/pull/95/)

# Version 1.5.1

This release brings support for standalone installations into Nix, as well as corrections for certain unusual edge case behaviors.

- Added a Nix flake, which allows for installing typeshare outside of NixPkgs
- `typeshare-core`
  - Now supports types in inline modules. [#109](https://github.com/1Password/typeshare/pull/109)
  - Now throws an error if `#[serde(flatten)]` is used, instead of silently generating incorrect types [#108](https://github.com/1Password/typeshare/pull/108)
  - When generating the `CodableVoid` type in swift, we now always include the `Codable` decorator, even if it's omitted from the list of `default_decorators` in `typeshare.toml` [#107](https://github.com/1Password/typeshare/pull/107)

### Community contributors

Thank you to the following community contributors for your work on this release:

- [nguarracino](https://github.com/nguarracino)

# Version 1.5.0

This release brings support for fixed-length arrays and fixes some premature changes made in 1.4.0 involving the
representation of unit types and enum variants in Typescript.

- `typeshare-core`
  - Fixed-length arrays in the form of `[T; N]` are now supported.
  - Reverted changes made to the representation of unit types in Typescript.<sup>1</sup>

<sup>1</sup> We apologize for the premature changes to the representation of unit types. This was done to improve
correctness and compatibility with various JSON libraries, however it ended up causing a regression by invalidating
certain usages. For now, we've reverted the changes so your projects can get back on track, and we are working to bring
these improvements while mitigating the issues discovered. Thank you for bringing this to our attention.

### Community contributors

Thank you to the following community contributors for your work on this release:

- [ccouzens](https://github.com/ccouzens)

# Version 1.4.0

This release brings topological sorting of types based on dependencies to generated files, as well as fixes several bugs.

- `typeshare-core`
  - Types are now outputted in order of dependency - types depending on others will be written after those without dependencies.
  - Unit types are now represented as null in Typescript
  - Deserialization for optional associated types in enum variants has been fixed.

### Community contributors

Thank you to the following community contributors for your work on this release:

- [adriangb](https://github.com/adriangb)

# Version 1.3.0

This release brings minor changes to snapshot testing by adding an additional option to remove version headers from generated code. This will make our snapshot tests more robust by preventing test breakage that used to occur when updating our version.

- `typeshare-core`
  - Each language implementation now has an additional public variable that can be set to remove version headers from generated code.

# Version 1.2.0

This release brings Scala functionality to the CLI, support for Apple Silicon as a pre-built binary, and refactors how
we handle language variants internally to be more type-safe.

- `typeshare-cli`
  - Scala is now a language generation target! Try it out with `typeshare --lang=scala --scala-package=com.your.package.here some/file.rs`
  - Future releases (including this one) now support aarch64-apple-darwin as an additional architecture.
- `typeshare-core`
  - Language variants are now represented as enums instead of strings.

### Community contributors

Thank you to the following community contributors for your work on this release:

- [jclmnop](https://github.com/jclmnop)
- [oeb25](https://github.com/oeb25)
- [DuhPesky](https://github.com/DuhPesky)
- [exoego](https://github.com/exoego)

# Version 1.1.0

This release brings major new additions, the largest of which is support for Scala as a language generation target.
Additionally, the code generation API has been expanded/revised, and many bugs have been fixed.

- `typeshare-cli`

  - Kotlin now uses `val` consistently for defining fields.
  - Some issues with the command line options have been corrected.
  - Unit structs that don't use bracket syntax are now supported.
  - Typescript can now handle type aliases of optional types.
  - Empty structs are now represented as objects in Kotlin.
  - You can now define read-only Typescript properties with `#[typeshare(typescript(readonly))]`.
  - Doubly-nested option types (`Option<Option<T>>`) are now supported in Typescript.

- `typeshare-core`

  - The `Language` trait now takes `self` mutably for more flexibility in implementations.
  - Scala is now a supported language for code generation, though the CLI does not use it yet.
  - The attribute parser has been reworked to be more robust and flexible for future additions.

- Miscellaneous
  - We now have a proper release system and prebuilt binaries for anyone to download 🎉
  - Releases will be weekly on every Thursday.

### Community Contributors

Thank you to the following community contributors for your work on this release:

[exoego](https://github.com/exoego), [Czocher](https://github.com/Czocher), [ccouzens](https://github.com/ccouzens),
[McAJBen](https://github.com/McAJBen), [adriangb](https://github.com/adriangb), [kareid](https://github.com/kareid),
[nihaals](https://github.com/nihaals), [ChrisMcKenzie](https://github.com/ChrisMcKenzie), [justintime4tea](https://github.com/justintime4tea),
[prestontw](https://github.com/prestontw), and [julienfouilhe](https://github.com/julienfouilhe)!
