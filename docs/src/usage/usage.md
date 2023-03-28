# Usage

This section will cover how to use Typeshare to generate type definitions for your project and how to interact with them across FFI. 

Typeshare provides a CLI tool that will perform the necessary generation for any types you annotate in your Rust code.

To generate ffi definitions for a specific target language, run the `typeshare` command and specify the directory containing your rust code, the language you would like to generate for, and the file to which your generated definitions will be written:
```
typeshare ./my_rust_project --lang=kotlin --output-file=my_kotlin_definitions.kt
typeshare ./my_rust_project --lang=swift --output-file=my_swift_definitions.swift
typeshare ./my_rust_project --lang=typescript --output-file=my_typescript_definitions.ts
typeshare ./my_rust_project --lang=scala --output-file=my_scala_definitions.scala
```
The first command-line argument is the name of the directory to search for Rust type definitions. The CLI will search all files in the specified directory tree for annotated Rust types. In addition to the input directory, you will also need to specify your desired target language and the output file to which the generated types will be written. This is done with the `--lang` and `--output-file` options respectively.

The currently supported output languages are:

- Kotlin
- Typescript
- Swift
- Scala
- Go

---
If your favourite language is not in this list, consider opening an issue to request it or try implementing it yourself! See our [contribution guidelines](../contributing.md) for more details.

---

In the following sections, we will learn how to customize the behaviour of Typeshare using the provided `#[typeshare]` attribute and configuration options.
