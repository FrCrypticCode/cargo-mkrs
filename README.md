# cargo-mkrs

## Cargo subcommand for generating Rust files

mkrs automatically generates module tree stuff and pastes a file header.

> [!WARNING]
> This is an experimental application. It may or may not do catastrophic damage to you project directory.

## Usage

Example:

```sh
cargo mkrs src/foo/bar              # Generate src/foo/bar.rs
cargo mkrs src/foo/mod --public     # Generate src/foo/mod.rs (declares `pub mod bar`)
cargo mkrs src/main                 # Generate src/main.rs (declares `mod foo`)
```

You can customize the template header by making a `.mkrs` (toml) file in the working directory:

```toml
# .mkrs
header = "/* This is an example header */"
```

That's about it.
