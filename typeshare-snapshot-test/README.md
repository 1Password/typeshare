# Typeshare Snapshot Test

`typeshare-snapshot-test` is a tool that helps you maintain your typeshare language implementations with snapshot tests. It is capable of capturing a series of snapshots for your typeshare binary, and then testing that the output of future runs continues to precisely match those snapshots.

## Test layout

Your snapshot tests should all live in a single directory, where each test is its own directory:

```
snapshot-tests/
  test1/
  test2/
  test3/
  multi-file-test/
```

Each test will need to provide some input, in the form of `.rs` rust files. Typically there will only be one file, but you can add as many as you like

```
snapshot-tests/
  test1/
    input.rs
  test2/
    input.rs
  test3/
    foo.rs
    bar.rs
  multi-file-test/
```

If you want to test multi-file output, you'll need a directory called `input`, which should contain a series of fake "crates" that will form the basis of the multi-file output:

```
snapshot-tests/
  test1/
    input.rs
  test2/
    input.rs
  test3/
    foo.rs
    bar.rs
  multi-file-test/
    input/
      crate1/
        src/
          lib.rs
          foo.rs
      crate2/
        src/
          lib.rs
```

## Generating snapshots

Once you have your initial set of tests, and whenever you need to add new tests or behaviors, you'll need to generate a snapshot. In this example we'll suppose we have an implementation of typeshare for the Go programming language.

```bash
typeshare-snapshot-test \
    # Each typeshare has a different name; this is the path to the actual
    # typeshare binary
    --typeshare typeshare-golang \

    # The language we're testing. Obviously, this language needs to be
    # supported by your particular typeshare.
    --language golang \
    --suffix .go \

    # We're generating new snapshot tests
    --mode generate

    # Path to the directory containing all the tests
    ./snapshot-tests
```

This will run a separate instance of your typeshare for each test in the tests directory. The output will be captured to `output.go` in single-file mode, or a directory called `golang` in multi-file mode:

```
snapshot-tests/
  test1/
    input.rs
+   output.go
  test2/
    input.rs
+   output.go
  test3/
    foo.rs
    bar.rs
+   output.go
  multi-file-test/
+   golang/
+     crate1.go
+     crate2.go
    input/
      crate1/
        src/
          lib.rs
          foo.rs
      crate2/
        src/
          lib.rs
```

## Running tests

Once you have snapshot tests, set up your continuous integration to test that
your output continues to produce the correct output. The command is the same
as with generating them, but with `--mode test`.

## Template Tests

In the future, we'll provide a utility that makes this step easy; for now, we recommend manually copying all of the tests from `app/cli/snapshot-tests` and generating your own snapshots.
