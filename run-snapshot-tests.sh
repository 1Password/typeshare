#!/bin/bash
# This bash script will run all our snapshot tests using the
# new snapshot test runner. The test runner only runs for a
# single output language so this script loops through all the
# supported languages. The test runner requires a pre-built
# typeshare binary to run, so this script starts by building
# a release profile of the binary.

# Test runner.
TEST="cargo run --release --bin typeshare-snapshot-test --"
# Location of our snapshot tests.
TEST_FOLDER="app/cli/snapshot-tests"
# Precompiled typeshare binary
TYPESHARE="target/release/typeshare2"
# Associative array of languages and filename extensions for each
# test runner iteration.
declare -A languages=(
    ["swift"]=".swift"
    ["typescript"]=".ts"
    ["kotlin"]=".kt"
)

cargo build --release --all-targets --bin typeshare2 && \
for lang in "${!languages[@]}"; do
    printf "Running snapshot tests for language %s\n" "$lang"
    $TEST -t $TYPESHARE --language "$lang" --mode test --suffix "${languages[$lang]}" $TEST_FOLDER
    # Break on first failure and return the failed status to the caller
    status=$?
    if [ $status -ne 0 ]; then
        printf "Test failed\n"
        exit $status
    fi
done

printf "All snapshot tests have passed\n"
