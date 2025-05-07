# typeshare-engine

This crate includes all of the actual implementations of typeshare functionality. It should be a dependency of anyone trying to USE typeshare as a library. It exports functions that make use of the trait in typeshare-model.

Currently, the public API of typeshare-engine is considered fairly unstable. Feel free to use it, but we expect updates to usually be published as major versions. You only need to depend on `typeshare-engine` if you want to use typeshare as a _library_; if you're implementing your own language, you only need to depend on [typeshare-model](../model), and if you're creating a typeshare binary, you only also need [typeshare-driver](../driver)
