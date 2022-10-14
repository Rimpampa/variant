This repo contains the implementation of the `variants!` macro, which is very minimal as it consists of
a single macro-by-example.

# Why use it?

Sometimes a lot of boilerplate needs to be written, and, be it because of some feature missing
in Rust itself or on some project specific restrictions, it cannot be avoided.

Most of the times Rust programmers spend some time to write `macros!` to reduce the amount of duplicate code,
but it doing so they expose themseleves to very criptic errors arising from macro expansion and worst of all
they lose the help of the linter, because a macro cannot be checked until it's called.

The `variants!` marco can be used for many such cases with the advantage that the code can be seen directly by
the linter and there won't be any macro expansion error (so long that the caller follows the described syntax)

# Usage

Add `variant` to your `Cargo.toml` file and start using it:
```toml
variant = "0.1.0"
```

Examples can be found in the documentation.

# Idea

This idea originated from a problem that a member of the Rust Italia telegram group was experiencing and for which I invented this macro.
The gist of the problem is summarized on the first example in the docs of `variants!`.
