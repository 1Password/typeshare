# Typeshare app

This directory contains all the crates related to the typeshare _binary_: the program that actually scans your rust code and generates types.

## Making your own typeshare

If you want to implement a typeshare for your own language, you've come to the right place. You only need two crates to do this:

- `typeshare-model`: this crate exports the core `Language` trait, along with some related types. The main thing you'll have to do is implement this trait for your language; see the [rustdocs](https://docs.rs/typeshare-model/latest/typeshare_model/trait.Language.html) for details on how to do this.
- `typeshare-driver`: this crate takes care of turning your `Language` implementation into a binary program. Using it is trivial: just add a `main.rs` with `typeshare_driver::typeshare_binary! { MyLanguage }`, and it will take care of the rest.

## Testing your typeshare

Once you have a typeshare binary you like, you'll want to test it. We have a tool called `typeshare-snapshot-test` that can help with this; it can use your typeshare to produce and then test snapshots with particular rust source input. See its documentation for details.
