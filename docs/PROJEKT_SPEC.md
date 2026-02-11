# GERMANIC – Project Specification

> Schema-driven compilation framework: JSON → .grm (typed binary for AI-readable websites)

## 1. Vision

Every website serves a small `.grm` binary alongside its HTML. AI systems read it instead of guessing from HTML. GERMANIC is the compiler that produces these files.

## 2. Architecture Overview

```
USER WORKFLOW (no Rust, no FBS knowledge needed):

  example.json ──► germanic init ──► restaurant.schema.json (editable)
                                              │
  data.json ──────────────────────► germanic compile ──► data.grm
                                              │
                                        (internally)
                                     Parse schema.json
                                     Validate data.json
                                     Build FlatBuffer dynamically
                                     Write .grm header + payload
```

### Two Modes

```
MODE 1: CLI Tool (for everyone)              MODE 2: Rust Library (for Rust devs)
═══════════════════════════════              ════════════════════════════════════
germanic init --from example.json            #[derive(GermanicSchema)]
germanic compile --schema X --input Y        struct PracticeSchema { ... }
                                             compile(&schema)?;
Runtime validation                           Compile-time type safety
No Rust knowledge needed                     Full trait-based contracts
Dynamic FlatBuffer builder                   Generated FlatBuffer bindings
```

### .grm File Format

```
Offset │ Size  │ Content
───────┼───────┼─────────────────────────────────────────
0x00   │ 3     │ Magic: "GRM" (0x47 0x52 0x4D)
0x03   │ 1     │ Version: 0x01
0x04   │ 2     │ Schema-ID length (little-endian u16)
0x06   │ n     │ Schema-ID (UTF-8, e.g. "de.health.practice.v1")
0x06+n │ 64    │ Ed25519 signature (or 64x 0x00)
...    │ ...   │ FlatBuffer Payload
```

FlatBuffers is the engine. `.grm` is the vehicle. The header makes anonymous bytes self-describing.

## 3. Current State (as of 2026-02-11)

### What exists and works (83 passing tests):
- Three-phase compiler pipeline: JSON → Rust Struct → FlatBuffer → `.grm`
- Procedural derive macro `#[derive(GermanicSchema)]` (darling-based)
- Traits: `SchemaMetadata`, `Validate`, `GermanicSerialize` (English)
- Recursive nested struct validation with error paths
- CLI: compile, init, validate, inspect, schemas commands
- FlatBuffers cross-namespace bug workaround (build.rs post-processing)
- **Dynamic compilation (Weg 3):** `germanic init` + `germanic compile --schema X.json`
  - Schema inference from example JSON
  - Runtime FlatBuffer builder (byte-compatible with flatc)
  - Validation against schema definitions
- **JSON Schema Draft 7 adapter:** Auto-detection, transparent dual-format support
- **All code in English** (German only in schema field names, schema IDs, project name)
- **42 domain schemas** proving universality across 11 categories
- **47 English + 42 German schemas** with fictional example data
- One static schema: Praxis (healthcare practice)

### What doesn't exist yet:
- README (personalized)
- Benchmark JSON-LD vs .grm
- OpenClaw SKILL.md
- Cargo.toml cleanup for crates.io
- MCP Server
- crates.io publication

## 4. Architecture Decision Records (ADRs)

### ADR-001: Translate identifiers to English
- **Status:** ✅ IMPLEMENTED
- **Context:** Code is entirely in German (function names, types, errors, comments). This blocks international adoption.
- **Decision:** Translate all Rust identifiers, comments, and docs to English. Keep German ONLY in:
  - FlatBuffer schema field names (e.g. `praxis.fbs` keeps `strasse`, `plz` etc.) — this is the brand
  - Schema IDs (e.g. `de.health.practice.v1`)
  - The project name GERMANIC
- **Executor:** Claude Code (Sonnet) with translation mapping

### ADR-002: Schemaless compilation (Weg 3)
- **Status:** ✅ IMPLEMENTED
- **Context:** Currently adding a new schema requires ~2.5h manual work including hand-written FlatBuffer serialization code. Users need Rust knowledge.
- **Decision:** Implement dynamic compilation mode where:
  1. `germanic init --from example.json` infers a schema definition
  2. User edits the `.schema.json` (marks required fields etc.)
  3. `germanic compile --schema X.schema.json --input data.json` produces .grm
  4. No Rust code, no .fbs files needed by end user
- **Key component:** Dynamic FlatBuffer builder that works from schema definitions at runtime
- **Trade-off:** Runtime validation instead of compile-time. Acceptable for CLI mode.
- **Implementation:** `dynamic/` module — schema_def.rs, infer.rs, builder.rs, validate.rs

### ADR-003: Keep existing static mode (Mode 1 + Mode 2)
- **Status:** ✅ IMPLEMENTED
- **Context:** The trait-based Rust library (`#[derive(GermanicSchema)]`) is architecturally valuable.
- **Decision:** Both modes coexist. Static mode stays for Rust devs, dynamic mode is added for everyone else.

### ADR-004: OpenClaw Skill + MCP Server
- **Status:** DECIDED (not yet implemented)
- **Context:** OpenClaw has 100k+ GitHub stars and a skill system based on SKILL.md files. GERMANIC as a CLI tool is a natural fit.
- **Decision:** Three integration levels:
  1. OpenClaw SKILL.md (teaches agent to use germanic CLI)
  2. MCP Server (universal tool integration for Claude, ChatGPT, etc.)
  3. ClawHub publication (discoverability)

### ADR-005: Fictional example data
- **Status:** ✅ IMPLEMENTED
- **Context:** Current examples contain real patient/practice data.
- **Decision:** Replace with clearly fictional examples. Keep same schema structure.
- **Implementation:** 42 German + 47 English fictional examples across 11 domain categories

### ADR-006: Dual license MIT OR Apache-2.0
- **Status:** ✅ IMPLEMENTED
- **Context:** Standard for Rust ecosystem. Maximizes adoption.
- **Decision:** MIT OR Apache-2.0 throughout.

### ADR-007: German code → English: Commit to the bit for schema names
- **Status:** ✅ IMPLEMENTED
- **Context:** The project is called GERMANIC. German schema field names are the brand.
- **Decision:** README says "Yes, the schema fields are in German. Because if you're going to be GERMANIC, commit to the bit." Code infrastructure is English, domain data stays German.

### ADR-008: JSON Schema Draft 7 Adapter
- **Status:** ✅ IMPLEMENTED
- **Context:** OpenClaw llm-task and the broader ecosystem speak JSON Schema Draft 7. Without compatibility, GERMANIC is isolated.
- **Decision:** Add a second "entry door" — an adapter that converts JSON Schema Draft 7 to SchemaDefinition. Auto-detection makes the switch transparent.
  - Adapter, NOT replacement: Our IndexMap-ordered format stays the internal truth.
  - JSON Schema `properties` → ordered fields, `required` list → per-field flags.
  - Unsupported features ($ref, anyOf, oneOf, allOf, enum) emit warnings, not errors.
  - `germanic compile --schema draft7.json data.json` works transparently.
- **Implementation:** `dynamic/json_schema.rs` — convert_json_schema(), is_json_schema(), load_schema_auto()

## 5. Translation Mapping (German → English) — ✅ EXECUTED

### Files renamed:
```
fehler.rs       → error.rs
schema.rs       → schema.rs (same, but contents translated)
schemas.rs      → schemas.rs (same)
schemas/praxis.rs → schemas/practice.rs
types.rs        → types.rs (same, but contents translated)
compiler.rs     → compiler.rs (same, but contents translated)
validator.rs    → validator.rs (same, but contents translated)
generated.rs    → generated.rs (same)
```

### Public API symbol mapping:
```
# Types / Structs / Enums
GermanicFehler          → GermanicError
ValidierungsFehler      → ValidationError
KompilierungsFehler     → CompilationError
HeaderParseFehler       → HeaderParseError
GrmHeader               → GrmHeader (unchanged)
SchemaTyp               → SchemaType
PraxisSchema            → PracticeSchema
AdresseSchema           → AddressSchema

# Enum variants
GermanicFehler::Validierung         → GermanicError::Validation
GermanicFehler::Json                → GermanicError::Json
GermanicFehler::Io                  → GermanicError::Io
GermanicFehler::UnbekanntesSchema   → GermanicError::UnknownSchema
GermanicFehler::Allgemein           → GermanicError::General
ValidierungsFehler::PflichtfelderFehlen → ValidationError::RequiredFieldsMissing
ValidierungsFehler::TypFehler       → ValidationError::TypeError
ValidierungsFehler::ConstraintVerletzung → ValidationError::ConstraintViolation
HeaderParseFehler::ZuWenigDaten     → HeaderParseError::InsufficientData
HeaderParseFehler::FalscheMagicBytes → HeaderParseError::InvalidMagicBytes
HeaderParseFehler::UngueltigeSchemaId → HeaderParseError::InvalidSchemaId
SchemaTyp::Praxis                   → SchemaType::Practice

# Traits
SchemaMetadaten         → SchemaMetadata
Validieren              → Validate
GermanicSerialisieren   → GermanicSerialize
GermanicSchemaVollstaendig → GermanicSchemaComplete

# Trait methods
schema_id()             → schema_id() (unchanged)
schema_version()        → schema_version() (unchanged)
validiere()             → validate()
zu_bytes()              → to_bytes()

# Functions
kompiliere()            → compile()
kompiliere_json()       → compile_json()
kompiliere_datei()      → compile_file()
schreibe_grm()          → write_grm()
von_str()               → from_str()
von_bytes()             → from_bytes()
felder_liste()          → field_list()

# Struct fields (GrmHeader)
schema_id               → schema_id (unchanged)
signatur                → signature

# Struct methods
GrmHeader::neu()        → GrmHeader::new()
GrmHeader::signiert()   → GrmHeader::signed()
GrmHeader::groesse()    → GrmHeader::size()

# Constants
GRM_MAGIC               → GRM_MAGIC (unchanged)
GRM_VERSION              → GRM_VERSION (unchanged)
SIGNATUR_GROESSE        → SIGNATURE_SIZE

# Type alias
GermanicResult          → GermanicResult (unchanged, keep brand)

# CLI (main.rs)
Kommandos               → Commands
Kommandos::Compile      → Commands::Compile (unchanged)
Kommandos::Schemas      → Commands::Schemas (unchanged)
Kommandos::Validate     → Commands::Validate (unchanged)
Kommandos::Inspect      → Commands::Inspect (unchanged)
cmd_compile()           → cmd_compile() (unchanged)
cmd_schemas()           → cmd_schemas() (unchanged)
cmd_validate()          → cmd_validate() (unchanged)
cmd_inspect()           → cmd_inspect() (unchanged)

# Macro crate (schema.rs in germanic-macros)
SchemaOptionen          → SchemaOptions
FeldOptionen            → FieldOptions
TypKategorie            → TypeCategory
implementiere_germanic_schema() → implement_germanic_schema()
generiere_validierungen() → generate_validations()
generiere_default_felder() → generate_default_fields()
typ_kategorie()         → type_category()

# Test names: translate to English (test_xyz_abc pattern)

# Error messages: translate to English

# Doc comments: translate to English

# KEEP GERMAN (brand identity):
# - Schema field names in .fbs files (strasse, plz, ort, etc.)
# - Schema IDs (de.gesundheit.praxis.v1 etc.)
# - The word "germanic" everywhere
# - beispiele/ folder → examples/
```

## 6. Schema Definition Format (for Weg 3)

```json
{
  "schema_id": "de.dining.restaurant.v1",
  "version": 1,
  "fields": {
    "name": {
      "type": "string",
      "required": true
    },
    "cuisine": {
      "type": "string"
    },
    "rating": {
      "type": "float"
    },
    "tags": {
      "type": "[string]"
    },
    "address": {
      "type": "table",
      "fields": {
        "street": { "type": "string", "required": true },
        "city": { "type": "string", "required": true },
        "zip": { "type": "string" },
        "country": { "type": "string", "default": "DE" }
      }
    }
  }
}
```

Supported types: `string`, `bool`, `int`, `float`, `[string]`, `[int]`, `table` (nested).

### JSON Schema Draft 7 (alternative input, auto-detected)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "de.health.practice.v1",
  "type": "object",
  "required": ["name", "telefon"],
  "properties": {
    "name": { "type": "string" },
    "telefon": { "type": "string" },
    "schwerpunkte": { "type": "array", "items": { "type": "string" } }
  }
}
```

Auto-detected and converted to SchemaDefinition transparently. See ADR-008.

## 7. Module Architecture (dynamic/)

```
src/dynamic/
├── mod.rs              compile_dynamic(), compile_dynamic_from_values(), load_schema_auto()
├── schema_def.rs       SchemaDefinition, FieldDefinition, FieldType
├── builder.rs          build_flatbuffer() — runtime FlatBuffer construction
├── validate.rs         validate_against_schema() — runtime validation
├── infer.rs            infer_schema() — JSON → .schema.json inference
└── json_schema.rs      convert_json_schema(), is_json_schema() — Draft 7 adapter
```

Data flow:
```
JSON Schema Draft 7 ──► is_json_schema() ──► convert_json_schema() ─┐
                                                                     ├──► SchemaDefinition
GERMANIC .schema.json ──► serde_json::from_str() ──────────────────┘            │
                                                                                 │
data.json ──► validate_against_schema() ◄───────────────────────────────────────┘
                      │
                      ▼
              build_flatbuffer() ──► FlatBuffer bytes ──► GrmHeader + payload ──► .grm
```

## 8. Tech Stack

- **Language:** Rust 2024 Edition (1.92.0+)
- **Binary format:** FlatBuffers (zero-copy)
- **CLI:** clap 4.5
- **Serialization:** serde, serde_json (with `preserve_order`)
- **Ordered maps:** indexmap 2.7 (with `serde` feature)
- **Error handling:** thiserror (library), anyhow (binary)
- **Macro:** darling (attribute parsing), proc-macro2, quote, syn
- **Crypto:** ed25519-dalek (signatures)
- **License:** MIT OR Apache-2.0
- **Target:** macOS (Apple Silicon primary), Linux, Windows
