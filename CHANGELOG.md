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
