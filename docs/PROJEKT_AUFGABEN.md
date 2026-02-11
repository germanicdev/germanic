# GERMANIC – Task Breakdown

## Critical Path

```
Phase 1 (Translation) ──► Phase 2 (Fictional Data) ──► Phase 3 (Weg 3) ──► Phase 4 (Release)
     │                          │                           │                     │
  ✅ DONE                    ✅ DONE                     ✅ DONE              IN PROGRESS
```

---

## Phase 1: Code Translation (German → English) ✅ COMPLETE

**Goal:** All Rust identifiers, comments, docs, and error messages in English. German stays ONLY in schema field names (.fbs), schema IDs, and the GERMANIC brand.

**Executor:** Claude Code (Sonnet)

### Task 1.1: Rename files + translate error.rs ✅ DONE
- fehler.rs → error.rs
- GermanicFehler → GermanicError, ValidierungsFehler → ValidationError
- All error messages, doc comments, test names in English

### Task 1.2: Translate types.rs ✅ DONE
- SIGNATUR_GROESSE → SIGNATURE_SIZE
- GrmHeader: neu() → new(), signiert() → signed()
- HeaderParseFehler → HeaderParseError
- von_bytes() → from_bytes(), zu_bytes() → to_bytes()

### Task 1.3: Translate schema.rs (traits) ✅ DONE
- SchemaMetadaten → SchemaMetadata
- Validieren → Validate, validiere() → validate()
- GermanicSerialisieren → GermanicSerialize, zu_bytes() → to_bytes()
- GermanicSchemaVollstaendig → GermanicSchemaComplete

### Task 1.4: Translate compiler.rs ✅ DONE
- kompiliere() → compile(), kompiliere_json() → compile_json()
- kompiliere_datei() → compile_file(), schreibe_grm() → write_grm()
- SchemaTyp → SchemaType

### Task 1.5: Translate schemas/practice.rs ✅ DONE
- PraxisSchema kept (brand identity, still referenced), English docs
- AdresseSchema with English docs
- German serde field names kept (strasse, plz, etc.)

### Task 1.6: Translate validator.rs ✅ DONE
- validate_grm(), validate_json() with English docs/messages

### Task 1.7: Translate main.rs (CLI) ✅ DONE
- Kommandos → Commands
- All CLI help text, println! output, function internals in English

### Task 1.8: Translate lib.rs ✅ DONE
- English doc comments and ASCII architecture diagram
- Module declarations updated

### Task 1.9: Translate germanic-macros ✅ DONE
- Internal German names remain in non-public macro code (SchemaOptionen etc.)
- Generated code paths updated: ::germanic::schema::Validate, ::germanic::error::ValidationError

### Task 1.10: Translate tests + build.rs ✅ DONE
- beispiele/ → examples/
- All test names in English
- build.rs comments in English

---

## Phase 2: Fictional Example Data ✅ COMPLETE

**Goal:** Replace real practice data with fictional examples.

### Task 2.1: Create fictional examples ✅ DONE
- dr-sonnenschein.json, waldberg-heilpraxis.json etc.
- All real patient/practice data removed

### Task 2.2: Integrate 42 universality domains ✅ DONE (AP-4.6)
- 42 .schema.json files across 11 domain categories
- 42 fictional example JSON files
- Domains: Gesundheit, Gastronomie, Handwerk, Immobilien, Bildung, Oeffentlich, Wirtschaft, Mobilitaet, Vereine, Landwirtschaft, Recht

---

## Phase 3: Schemaless Compilation (Weg 3) ✅ COMPLETE

**Goal:** `germanic init` + `germanic compile` with .schema.json, no Rust/FBS knowledge needed.

### Task 3.1: Schema definition format + parser ✅ DONE
**Implemented in:** `dynamic/schema_def.rs`
- SchemaDefinition struct with schema_id, version, fields (IndexMap)
- FieldDefinition with field_type, required, default, nested fields
- FieldType enum: String, Bool, Int, Float, StringArray, IntArray, Table
- IndexMap preserves field order → deterministic vtable slot assignment
- from_file() / to_file() serialization

### Task 3.2: JSON type inference (init command) ✅ DONE
**Implemented in:** `dynamic/infer.rs`
- infer_schema(): Walks JSON value tree, infers types
- String, Bool, Int, Float, StringArray, IntArray, Table detection
- Null → String fallback
- All fields default to required: false (user edits manually)

### Task 3.3: Dynamic FlatBuffer builder ✅ DONE
**Implemented in:** `dynamic/builder.rs`
- build_flatbuffer(): Schema + JSON → FlatBuffer bytes at runtime
- Two-phase building: (1) pre-create offsets, (2) start table + push slots
- vtable slot formula: voffset = 4 + (2 * field_index)
- PreparedField enum: Absent, Offset, Bool, Int, Float
- Recursive nested table support
- **Byte-compatible with flatc output** (proven by integration tests)

### Task 3.4: Validation against schema definition ✅ DONE
**Implemented in:** `dynamic/validate.rs`
- validate_against_schema(): Checks required fields, types, nested structures
- Non-fail-fast: collects ALL violations before returning error
- Layer 1: Required fields present?
- Layer 2: Types match schema?
- Layer 3: Nested tables valid? (recursive)

### Task 3.5: CLI integration (init + compile dynamic mode) ✅ DONE
**Implemented in:** `dynamic/mod.rs` + `main.rs`
- compile_dynamic(): schema file + data file → .grm bytes
- compile_dynamic_from_values(): in-memory variant
- Auto-detection: .json extension → dynamic mode, named schema → static mode
- `germanic init --from example.json --schema-id X` → infers .schema.json
- `germanic compile --schema X.schema.json --input data.json` → .grm

### Task 3.6: End-to-end tests ✅ DONE
**Implemented in:** `tests/byte_compat_test.rs`
- test_dynamic_minimal_praxis: Minimal schema → .grm
- test_dynamic_praxis_readable_by_static_types: Dynamic output byte-compatible with flatc

---

## Phase 4: Release Preparation — IN PROGRESS

**Goal:** Public GitHub, crates.io, OpenClaw skill, benchmark, JSON Schema compatibility

### Task 4.1: README ⬜ TODO
**Dependencies:** Phase 1-2

### Task 4.2: Benchmark JSON-LD vs .grm ⬜ TODO
**Dependencies:** Phase 1

### Task 4.3: OpenClaw SKILL.md ⬜ TODO
**Dependencies:** Phase 3 (available)

### Task 4.4: Cargo.toml cleanup for crates.io ⬜ TODO
**Dependencies:** Phase 1

### Task 4.5: MCP Server ⬜ TODO
**Dependencies:** Phase 3

### Task 4.6: GitHub public + crates.io publish ⬜ TODO
**Dependencies:** All above

### Task 4.6 (AP): Universality Examples ✅ DONE
**Implemented:** 42 domain schemas + 42 examples across 11 categories
**AP document:** `docs/AP-4.6_UNIVERSALITAETS_BEISPIELE.md`

### Task 4.6b (AP): English Schema Translations ✅ DONE
**Implemented:** 47 English schemas + 52 English examples
- Reorganized: `examples/de/`, `examples/en/`, `schemas/definitions/de/`, `schemas/definitions/en/`
- Schema IDs: `en.health.pharmacy.v1`, `en.dining.restaurant.v1`, etc.
**AP document:** `docs/AP-4.6b_ENGLISCHE_UEBERSETZUNGEN.md`

### Task 4.7 (AP): JSON Schema Draft 7 Adapter ✅ DONE
**Implemented in:** `dynamic/json_schema.rs` (~230 lines + ~220 lines tests)
- convert_json_schema(): Draft 7 → SchemaDefinition
- is_json_schema(): Format detection heuristic
- load_schema_auto(): Transparent dual-format loader
- Full type mapping: string, boolean, integer, number, object, array
- Required-list inversion, nested objects, array item type inference
- Warnings for unsupported features ($ref, anyOf, oneOf, allOf, enum)
- CLI auto-detection: `--schema` accepts both formats transparently
- 23 unit tests including OpenClaw llm-task compatibility test
**AP document:** `docs/AP-4.7_JSON_SCHEMA_ADAPTER.md`
