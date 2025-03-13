# typeshare-engine

This crate includes all of the actual implmenentations of typeshare functionality. It should be a dependency of anyone trying to USE typeshare as a library. It depends ONLY on typeshare-model, not on any language crates. It exports functions that make use of the traits in typeshare-model, so that callers can do something like this (this is just an example, not my real intended API)

```rust
use std::env::current_dir;
use typeshare_engine::run;
use typeshare_swift::Swift;

fn main() {
    let working_dir = current_dir();
    let source_dir = working_dir.join("src");
    let out_dir = working_dir.join("output");

    run(source_dir, working_dir, Swift);
}
```
