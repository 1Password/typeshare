# typeshare modularization project

This document has been copied from the proposal in notion and serves as a quick reference. Notion should remain the source of truth, if possible.

- Basic outline:
    - Typeshare-macro: provides the macro itself. The macro is basically a no-op so not much is needed here. Can perform checks that the tagged type *can* be typeshared.
    - Typeshare-model crate: Provides traits, data model. Dependency of language implementations.
    - typeshare-engine: parsing, executing. This crate is a depedency of any library wishing to use typeshare as a library
    - typeshare-driver: provides a macro similar to `tokio::main`, which creates an entry point using a list of languages.

        ```rust
        generate_typeshare_cli!{Swift, Kotlin, Typescript}
        ```

    - language crates:
        - typeshare-typescript
        - typeshare-kotlin
        - typeshare-swift
        - etc
    - typeshare-cli will be a “blessed” set of languages supported by the PFT team. It will create a binary called typeshare that will hopefully be a drop-in replacement for the one we have now.
- The intended architecture is that each language crate will have both a library mode and a CLI mode, with a binary supporting just that one language. Additionally, we’ll publish a typeshare-cli crate, equivalent to the one we have now, which supports the languages our team uses, for compatibility and efficiency.
