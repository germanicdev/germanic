# GERMANIC

**Structured AI feeds & schemas. Fast indexing. Rust-first.**

GERMANIC is a protocol that makes websites machine-readable for AI systems.
It uses FlatBuffers for zero-copy serialization and provides type-safe schemas
for structured data.

## Installation
```bash
cargo install germanic
```

## Usage
```bash
# List available schemas
germanic schemas

# Compile JSON to .grm
germanic compile --schema restaurant --input data.json

# Inspect a .grm file
germanic inspect output.grm
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
