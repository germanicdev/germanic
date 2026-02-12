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

## The Contract Proof

> What happens when data is **wrong**?

Every row is a real test. Run `cargo test --test vertragsbeweis` to verify.

| # | Scenario | HTML/Scraping | JSON-LD | JSON Schema D7 | GERMANIC |
|---|----------|:---:|:---:|:---:|:---:|
| S1 | Required field missing | silent | silent | reports | **WON'T COMPILE** |
| S2 | Required field empty `""` | silent | silent | silent | **WON'T COMPILE** |
| S3 | Wrong type: `"ja"` instead of `true` | silent | silent | reports | **WON'T COMPILE** |
| S4 | Prompt injection in text field | executable | injectable | injectable | **binary bytes** |
| S5 | Nested field missing | silent | silent | reports | **WON'T COMPILE** |
| S6 | String where int expected | silent | silent | reports | **WON'T COMPILE** |
| S7 | Unknown extra field | absorbed | absorbed | accepted | **stripped** |
| S8 | `null` for required field | silent | silent | often silent | **WON'T COMPILE** |

> JSON Schema: `type: "string"` is satisfied by `""`. You'd need explicit `minLength: 1`.
> JSON Schema: Nested `required` only works if correctly defined.
> JSON Schema: `additionalProperties` defaults to `true`.
> JSON Schema: Many implementations treat `null` laxly with `type: "string"`.
>
> **Measured on:** `cargo test --test vertragsbeweis` — not benchmarks, but guarantees.
> **Source:** [`tests/vertragsbeweis.rs`](crates/germanic/tests/vertragsbeweis.rs)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
