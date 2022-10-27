This repo is very minimal as it consists of a single macro-by-example which is `variants!`.

This create does basically the same thing as [`duplicate`](https://docs.rs/duplicate/latest/duplicate/),
which uses a different syntax and it's implemented using a procedural macro.

# Usage

Add `variant` to your `Cargo.toml` file and start using it:
```toml
variant = "0.1.0"
```

Examples can be found in the documentation.

# Idea

This idea originated from a problem that a member of the Rust Italia telegram group was experiencing and for which I invented this macro.
The gist of the problem is summarized on the second example in the docs of `variants!`.
