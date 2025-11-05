# Making your own typeshare

Typeshare is designed to make it easy to implement your own languages as a separate binary. This document provides a basic walkthrough for how to create your own langauge implementation.

## Project Structure

There are two key components to making your own Typeshare implementation:
- `typeshare-model`
- `typeshare-driver`

In a typical Typeshare project, each language implementation will be its own crate, with the `typeshare-driver` bringing your implementations together into a single CLI binary.

```
typeshare-binary
├── typeshare-language-1
│   ├── src
│   │   └── lib.rs
│   └── cargo.toml
├── typeshare-language-2
│   ├── src
│   │   └── lib.rs
│   └── cargo.toml
├── src
│   └── main.rs
└── cargo.toml
```

## Start the Project

Set up the binary:

```bash
cargo new my-typeshare-binary
cd my-typeshare-binary
```

This is where the `typeshare-driver`'s main macro for setting up the binary will be. Add the `typeshare-driver` dependency
```bash
cargo add typeshare-driver
```
Add a crate for your language implementation (or as many as you want! Up to 16):
```bash
cargo new my-typeshare-language --lib
cd my-typeshare-language
```
Once you have your language crate, add the `typeshare-model` dependency:

```bash
cargo add typeshare-model
```
Some other dependencies you will need to implement your own language include:
- `thiserror`
- `serde` with the feature `derive`
- `joinery`
- `itertools`
- `anyhow`

So make sure to add these to your language crate as well

## Implementing a Language

Once your project is set up, you need to implement the Language trait from `typeshare-model` inside your langugae crate.

There a few functions that have to be implemented: 
- `new_from_config`: This instantiates your language struct using configuration from a `typeshare.toml` file or the command line
- `output_filename_for_crate`: This is used in multi-file mode for setting up consistent naming for the files.
- `write_*`: These functions are your implementations of how various types should be handled. These implementations should typically call `format_type`, which is used to format Rust types into strings for writing to the Typeshare generated file.
- `format_special_type`: This is where you add custom implementations of special types for each language that is called from within `format_type`.

Additional functions that are optional but common to implement include:
- `mapped_type`: This allows you to create specific custom handling for specific types. 
- `begin_file`, `end_file`, `write_additional_files` to add other per-file or per-directory handling, such as information about the file generation in comments at the top of the file, or other custom handling.

See the function documentation for more detailed information and look through our implementations of Typeshare for [Kotlin](../../app/langs/kotlin/), [Swift](../../app/langs/swift/), and [Typescript](../../app/langs/typescript/) for examples.

Once the Language trait has been implemented, your Typeshare is ready to be built!

## Building Your Typeshare

In `main.rs` once your language has been implemented, all you have to do is pass them into the `typeshare_driver` macro:
```rust
use typeshare_driver::typeshare_binary;
use typeshare_language_1::YourLanguage1;
use typeshare_language_2::YourLanguage2;

typeshare_binary! { YourLanguage1, YourLanguage2 }
```
Now, running `cargo build` will build your Typeshare with your language implementations and create a fully functional CLI tool.

## Using Your Typeshare

In a project where you want to use Typeshare, add `typeshare` to your `cargo.toml` and add the Typeshare annotations to any types you want to share (see more about using annotations [here](./usage/annotations.md)). Set up a `typeshare.toml` at the root of your project with any configuration information for your implementation. 

The general usage of a Typeshare CLI binary is:
```bash
typeshare-binary --lang <your-language-1>  --output-file <path-to-your-output-file> <path-to-directory-to-run-typeshare-on>
```
or
```bash
typeshare-binary --lang <your-language-1>  --output-folder <path-to-your-output-folder> <path-to-directory-to-run-typeshare-on>
```
depending if you want the generated code to be outputted into a folder or a file.

To run your Typeshare for multiple languages, you rerun the command with each desired language.

To see all possible languages in a Typeshare and learn about other possible commands run:
```bash
typeshare-binary --help
```