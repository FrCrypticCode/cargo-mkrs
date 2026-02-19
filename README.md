# cargo-mkrs

Cargo subcommand for generating Rust files

> [!WARNING]
> This is an experimental application. It may or may not do catastrophic damage to you project directory.

# Usage

```sh
cargo mkrs src/foo/bar              # Generate src/foo/bar.rs
cargo mkrs src/foo/mod --public     # Generate src/foo/mod.rs (declares `pub mod bar`)
cargo mkrs src/main                 # Generate src/main.rs (declares `mod foo`)
```

That's about it.
