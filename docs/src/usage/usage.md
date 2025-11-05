# Usage

This section will cover how to use Typeshare to generate type definitions for your project and how to interact with them across FFI.

A Typeshare binary provides a CLI tool that will perform the necessary generation for any types you annotate in your Rust code.

There are a few options for where to find various Typeshare language implementations. You can implement your own (see [here](../new-languages.md)), use language implementations by other open source developers (eg. [typeshare-java](https://crates.io/crates/typeshare-java)), or use 1Password's Typeshare2 CLI.

The currently supported output languages in the 1Password provided Typeshare2 CLI are:

- Kotlin
- Typescript
- Swift

To generate ffi definitions for a specific target language, run the CLI command for the Typeshare you are using. Specify the directory containing your rust code, the language you would like to generate for, and the file to which your generated definitions will be written. For example, with 1Password's Typeshare CLI:

```
typeshare2 ./my_rust_project --lang=kotlin --output-file=my_kotlin_definitions.kt
typeshare2 ./my_rust_project --lang=swift --output-file=my_swift_definitions.swift
typeshare2 ./my_rust_project --lang=typescript --output-file=my_typescript_definitions.ts
```

The first command-line argument is the name of the directory to search for Rust type definitions. The CLI will search all files in the specified directory tree for annotated Rust types. In addition to the input directory, you will also need to specify your desired target language and the output file to which the generated types will be written. This is done with the `--lang` and `--output-file` options respectively.

---

If your favourite language is not in this list, try implementing it yourself! See how in our [language implementation guide](../new-languages.md) for more details.
