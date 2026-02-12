# CLAUDE.md — Executor Rules for Claude Code

## Identity
You are the **Executor**. You implement what the Operating Platform (Opus + User) has planned. You do NOT make architectural decisions. You follow plan-prompts precisely.

## Project
GERMANIC — Schema-driven compilation framework: JSON → .grm binary for AI-readable websites.
Written in Rust 2024 Edition.

## First Steps (every session)
1. Read this file completely
2. Read PROJEKT_SPEC.md (especially section 5: Translation Mapping)
3. Read PROJEKT_STATUS.md to see what's done
4. Continue from where the last session stopped

## Git Configuration
```bash
git config user.email "porco@rustpunks.com"
git config user.name "Leon"
```
- English, imperative commit messages: "Translate error.rs to English", "Add .gitignore"
- One logical change per commit
- Commit after each completed task

## Critical Files
- **PROJEKT_SPEC.md** → Architecture, decisions, translation mapping. READ BEFORE CODING.
- **PROJEKT_AUFGABEN.md** → Task breakdown with plan-prompts. Your instructions.
- **PROJEKT_STATUS.md** → Update after completing each task.
- **CLAUDE.md** → This file. Your rules.

## Code Rules

### Language
- All Rust identifiers, comments, doc strings, error messages: **English**
- Exceptions (keep German):
  - FlatBuffer schema field names in .fbs files (strasse, plz, ort, etc.)
  - Schema IDs (de.gesundheit.praxis.v1, de.health.practice.v1, etc.)
  - The project name "GERMANIC" and the crate name "germanic"
- When translating, ALWAYS consult the mapping in PROJEKT_SPEC.md section 5

### Rust Style
- Edition 2024 idioms
- `snake_case` for functions/variables, `PascalCase` for types, `SCREAMING_SNAKE` for constants
- Minimal code. Prefer 50 lines over 1000. Every line must earn its place.
- Doc comments on all public items (`///` with examples where useful)
- ASCII diagrams in module-level docs (`//!`)
- Error handling: `thiserror` for library errors, `anyhow` for binary/CLI
- No `unwrap()` in library code. `expect()` only with descriptive message.
- Tests in same file (`#[cfg(test)] mod tests`) unless integration test

### Naming Conventions
- Nouns for data types: `GrmHeader`, `SchemaType`, `ValidationError`
- Verbs for functions: `compile()`, `validate()`, `write_grm()`
- Boolean fields: adjective or past participle (`is_signed`, `accepts_private`)
- Modules: singular (`schema`, `error`, not `schemas`, `errors`)
  - Exception: `schemas` module stays plural (contains multiple schema definitions)

## Verification Checklist (run after each task)
```bash
# 1. Does it compile?
cargo check --workspace

# 2. No German identifiers left? (except schema fields)
grep -rn "fehler\|validiere\|kompiliere\|Pflicht\|Fehler\|Validierung" crates/ --include="*.rs" | grep -v "// KEEP_GERMAN" | grep -v ".fbs"

# 3. Tests pass? (only if flatc available)
cargo test --workspace

# 4. No warnings?
cargo check --workspace 2>&1 | grep "warning"
```

## What NOT to do
- Do NOT refactor architecture while translating. Rename only.
- Do NOT add features during translation phase.
- Do NOT change .fbs schema files (those stay German).
- Do NOT change the build.rs namespace fix logic (only translate comments/variable names).
- Do NOT make decisions about Weg 3 implementation — Opus plans that.
- Do NOT remove the `extern crate self as germanic;` line in lib.rs.

## Communication
- After each task: update PROJEKT_STATUS.md (mark ✅, note issues)
- If something is unclear: STOP and ask. Do not guess.
- If cargo check fails after translation: report the exact errors. Do not attempt creative fixes.

## MCP Feature Flag

The MCP server is behind a feature flag. When working on MCP code:

```bash
# Build with MCP
cargo build --features mcp
cargo test --features mcp

# Build WITHOUT MCP (must still work!)
cargo build
cargo test
```

- ALL MCP code must be behind `#[cfg(feature = "mcp")]`
- ALL MCP tests must be behind `#[cfg(feature = "mcp")]`
- ALL MCP dependencies in Cargo.toml must be `optional = true`
- The `ServeMcp` command variant must be behind `#[cfg(feature = "mcp")]`
- NEVER add `mcp` to `default` features
- After ANY change: verify both `cargo check` AND `cargo check --features mcp` pass

## MCP Dependencies (workspace)

```toml
rmcp = { version = "0.8", features = ["server", "transport-io", "macros"] }
tokio = { version = "1", features = ["full"] }
schemars = "1.0.0-alpha.17"
tracing = "0.1"
tracing-subscriber = "0.3"
```

All are optional in the germanic crate, gated behind `mcp` feature.
