# Testing Your Typeshare

## Setting up Snapshot Testing for Typeshare

When you have a working Typeshare, you likely will want some way to validate that making changes to the implementation:
1. Does what you expect it to do and 
2. Doesn't alter previously expected behaviour

The way this is achieved in Typeshare is with snapshot testing. The `typeshare-snapshot-test` tool enables you to generate snapshots from your Typeshare's current state of type generation and test future changes against these snapshots. To install the CLI run:
```bash
cargo install typeshare-snapshot-test
```
This will install the tool globally. You can learn more about the crate [here](https://crates.io/crates/typeshare-snapshot-test)

## Generating Snapshots

To generate snapshot tests, first you need to set up your file structure. To set up your snapshot tests, make a directory where they will all be stored. 

Inside this directory, every subdirectory is a test. Tests can be structured with one or more rust input files. They can also be set up to support multi-file mode. Each test can be set up with its own config `typeshare.toml` file for test-specific configuration.
```
snapshot-tests
├── test-1
│   ├── input.rs
│   └── typeshare.toml
├── test-2
│   ├── input-1.rs
│   ├── input-2.rs
│   └── typeshare.toml
└── multi-file-test
    └── input
        ├── basic_crate
        │   └── src
        │       └── lib.rs
        └── dependent_crate
            └── src
                └── lib.rs
```
To generate snapshots, `typeshare-snapshot-test` will generate snapshots from this directory structure, outputing either an output file or a folder with the language's name if it is a multi-file test.

The basic command to geenrate snapshots is

```bash
typeshare-snapshot-test --typeshare <your-typeshare-binary> --language <your-typeshare-language> --suffix <your-language-suffix> --mode generate <path-to-tests>
```

To generate snapshots for multiple languages in your Typeshare, rerun the generation command for each language.


## Testing Against Snapshots

To run the snapshot tests, pass in `test` to the `--mode` flag instead of `generate`.

```bash
typeshare-snapshot-test --typeshare <your-typeshare-binary> --language <your-typeshare-language> --suffix <your-language-suffix> --mode test <path-to-tests>
```

This will run all the tests for the specified language. To run any snaphshot tests for other languages in your Typeshare, rerun the command for each language.

## Generating and Running Individual Tests

Sometimes you don't want to run all the tests in the snapshot test directory. `typeshare-snapshot-test` uses the test folders' names as the tests' names. Specific tests can be specified by name with the `--include` flag. Use this flag multiple times to run multiple tests.

```bash
typeshare-snapshot-test --typeshare <your-typeshare-binary> --language <your-typeshare-language> --suffix <your-language-suffix> --include <test-1> --include <test-2> --mode test <path-to-tests>
```

This can also be used to only generate new snapshots for specific tests instead of all the tests within the snapshot-tests directory.

```bash
typeshare-snapshot-test --typeshare <your-typeshare-binary> --language <your-typeshare-language> --suffix <your-language-suffix> --include <test-1> --include <test-2> --mode generate <path-to-tests>
```
A common usage of this is if your Typeshare has multiple languages. Some tests might be for specific languages or specific subsets of languages. 

For example:
```
snapshot-tests
├── test-go-and-ts
│   ├── input.rs
│   ├── output.ts
│   ├── output.go
│   └── typeshare.toml
├── test-1-ts-only
│   ├── input.rs
│   ├── output.ts
│   └── typeshare.toml
└── test-2-go-only
    ├── input-1.rs
    ├── output.go
    └── typeshare.toml
```
Using the include flag lets you create and run any combinations of tests with specific languages.

