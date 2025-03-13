# typeshare-core

The core library behind typeshare, containing type definitions, parsing, and code generation.

## Formatting

Formatting code is hard. Like, [really, really hard](http://journal.stuffwithstuff.com/2015/09/08/the-hardest-program-ive-ever-written/). Typeshare happens to be a program that needs to generate beautifully formatted code in multiple languages.

In the interests of avoiding the complexities of formatting code perfectly, typeshare takes the approach of generating "best-effort" output that it then runs through the appropriate formatting tool for each language. This makes typeshare's output exactly what language devs would expect, every time, with minimal effort on our part.

Generating well-formatted output therefore requires you to have the appropriate formatting tools for each language installed and available when running typeshare. If you don't have them, typeshare's default (and less pretty) output will be used.

## Testing

typeshare's test suite is built on the concept of snapshot testing, a data-driven testing methodology that aims to make it quick and painless to enact large, sweeping changes in a codebase.

Tests are declared inside of a macro defined in [`tests/snapshot_tests.rs`](tests/snapshot_tests.rs). Once a test has been declared, run:

```
env UPDATE_EXPECT=1 cargo test -p typeshare-core
```

This will generate the the folder for the new test (inside of [`data/tests`](data/tests)) along with starter files inside of it. Save whatever Rust source input you'd like to test in the `input.rs` file. Then, run the command again:

```
env UPDATE_EXPECT=1 cargo test -p typeshare-core
```

The various output files will be updated with typeshare's current output for the given input. If you're happy with the output, move on; if not, hack on typeshare until the output makes sense, re-running the above command each time you'd like to update the expected output.

The test suite can of course be run normally without updating any expectations:

```
cargo test -p typeshare-core
```

If you find yourself needing to update expectations for a specific test only, run the following (substituting the name of your test in for the last arg):

```
env UPDATE_EXPECT=1 cargo test -p typeshare-core --test snapshot_tests -- can_handle_serde_rename_all::swift
```

The data stored in the snapshot test files is typeshare's output, unmodified.

This will write `output_formatted.(ts|kt|swift)` files next to each stored snapshot file.
