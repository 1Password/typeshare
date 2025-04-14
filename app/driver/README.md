# typeshare-driver

This crate contains a macro generating all the glue code necessary to create a typeshare binary. Supposing you had your own Python and Golang implementations
of typeshare, all you need to write is this:

```rust
use typeshare_driver::typeshare_binary;

use typeshare_golang::Golang;
use typeshare_python::Python;

typeshare_binary! { Python, Golang }
```

This creates an `fn main` that uses these languages, plus `typeshare-engine`, to implements a full typeshare CLI.
