#!/bin/bash
TEST="cargo run --release --bin typeshare-snapshot-test --"
TEST_FOLDER="app/cli/snapshot-tests"
TYPESHARE="target/release/typeshare2"
declare -A languages=(["swift"]=".swift" ["typescript"]=".ts" ["kotlin"]=".kt")

cargo build --release --all-targets && \
for lang in "@{!languages[@]}"
do
    echo "Running $lang tests"
    $TEST -t $TYPESHARE --language "$lang" --mode test --suffix "${languages[$lang]}" $TEST_FOLDER
done
