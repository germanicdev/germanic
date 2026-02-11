# GERMANIC – Project Status

## Current Phase: Phase 1 — Code Translation + Phase 2 — Fictional Data
**Started:** 2025-02-11
**Target:** All identifiers, comments, docs in English. Real data replaced.

## Progress

### Phase 0: Repository Setup
| Task | Status | Notes |
|------|--------|-------|
| 0.0 Git init + anonymize | ✅ DONE | Commit b8cee52 |
| 0.1 .gitignore + git config | ⬜ TODO | porco@rustpunks.com |

### Phase 1: Translation
| Task | Status | Notes |
|------|--------|-------|
| 1.1 Rename files + error.rs | ⬜ TODO | fehler.rs → error.rs |
| 1.2 types.rs | ⬜ TODO | |
| 1.3 schema.rs (traits) | ⬜ TODO | |
| 1.4 compiler.rs | ⬜ TODO | Depends on 1.1-1.3 |
| 1.5 schemas/practice.rs | ⬜ TODO | Depends on 1.1-1.3 |
| 1.6 validator.rs | ⬜ TODO | |
| 1.7 main.rs (CLI) | ⬜ TODO | Depends on 1.4-1.5 |
| 1.8 lib.rs | ⬜ TODO | Depends on 1.1-1.7 |
| 1.9 germanic-macros | ⬜ TODO | Independent, can parallel |
| 1.10 tests + build.rs + cleanup | ⬜ TODO | Final verification |

### Phase 2: Fictional Data
| Task | Status | Notes |
|------|--------|-------|
| 2.1 Replace examples | ⬜ TODO | After Phase 1 |
| 2.2 Integrate 42 universality domains | ✅ DONE | 42 schemas + 42 examples |

### Phase 3: Schemaless Compilation (Weg 3)
| Task | Status | Notes |
|------|--------|-------|
| 3.1-3.6 | ⬜ TODO | Opus plans architecture first |

### Phase 4: Release
| Task | Status | Notes |
|------|--------|-------|
| 4.1-4.6 | ⬜ TODO | After Phase 3 |
| 4.6b English schema translations | ✅ DONE | 47 schemas + 52 examples |
| 4.7 JSON Schema Draft 7 Adapter | ✅ DONE | json_schema.rs + CLI auto-detect |

## Blockers
- flatc must be installed for cargo test/build (`brew install flatbuffers`)

## Decisions Made
1. Translate code to English (Sonnet executes)
2. Implement Weg 3 (schemaless compilation) — Opus plans
3. Target OpenClaw + MCP integration
4. Replace real data with fictional examples
5. Keep German schema field names as brand identity
6. Two-mode architecture: CLI for everyone + Rust library for devs
7. Consolidate Phase 1+2 into single Claude Code session

## Recent Completions
### 2025-02-11: English Schema Translations (AP-4.6b)
- ✅ Reorganized folder structure: `examples/de/`, `examples/en/`, `schemas/definitions/de/`, `schemas/definitions/en/`
- ✅ Translated all 47 German schemas to English with idiomatic English schema IDs:
  - Health: pharmacy, practice, hospital, careservice, psychotherapist, veterinarian, midwife, optician
  - Dining: restaurant, cafe, hotel, vacation-rental, campground, cinema, museum, pool
  - Trades: electrician, photographer, it-service, cleaning, locksmith
  - Real Estate: listing, rental-listing, energy-consultant, property-mgmt
  - Education: daycare, music-school, community-college, tutoring
  - Public: citizen-office, event, tourist-info, farmers-market
  - Business/Employment: job-posting, company, trade-fair
  - Mobility: charging-station, parking-garage, bike-rental
  - Associations: sports-club, church
  - Agriculture: farm-shop, winery
  - Legal: law-firm, tax-advisor
  - Services: craftsman
  - Events: public-event
- ✅ Translated all 52 example JSON files to English (field names + content)
- ✅ Maintained German cities, addresses, names, phone numbers (fictional data in Germany)
- ✅ Translated descriptions, array values (specializations, languages, etc.) to English
- ✅ Tested compilation with `germanic compile` for multiple English examples
- ✅ Verified schema IDs: `en.health.pharmacy.v1`, `en.dining.restaurant.v1`, `en.services.craftsman.v1`, etc.
- ✅ All English examples compile successfully with correct schema IDs

### 2025-02-11: Universality Examples Integration
- ✅ Extracted 42 domain schemas from GERMANIC_UNIVERSALITAETS_BEISPIELE.md
- ✅ Created `/examples/schemas/` directory with all 42 .schema.json files
- ✅ Created 42 fictional example JSON files covering 11 domain categories:
  - Gesundheit (7): apotheke, krankenhaus, pflegedienst, psychotherapeut, tierarzt, hebamme, optiker
  - Gastronomie (7): cafe, hotel, ferienwohnung, campingplatz, kino, museum, schwimmbad
  - Handwerk (5): schluesseldienst, elektriker, it-dienstleister, fotograf, reinigung
  - Immobilien (3): expose-miete, energieberater, hausverwaltung
  - Bildung (4): kita, musikschule, volkshochschule, nachhilfe
  - Oeffentlich (4): buergeramt, veranstaltung, touristeninformation, wochenmarkt
  - Wirtschaft (3): stellenanzeige, unternehmen, messe
  - Mobilitaet (3): ladesaeule, parkhaus, fahrradverleih
  - Vereine (2): sportverein, kirchengemeinde
  - Landwirtschaft (2): hofladen, winzer
  - Recht (2): anwaltskanzlei, steuerberater
- ✅ All files validated as proper JSON

### 2026-02-11: JSON Schema Draft 7 Adapter (AP-4.7)
- ✅ New module `dynamic/json_schema.rs` (~230 lines code + ~220 lines tests)
- ✅ `convert_json_schema()`: Converts JSON Schema Draft 7 → SchemaDefinition
- ✅ `is_json_schema()`: Auto-detects JSON Schema Draft 7 vs GERMANIC native format
- ✅ `load_schema_auto()`: Transparent schema loading with auto-detection
- ✅ Full type mapping: string, boolean, integer, number, object, array
- ✅ Required-list inversion (object-level → per-field flags)
- ✅ Nested objects → FieldType::Table (recursive)
- ✅ Array items type inference (string/integer arrays)
- ✅ Default value pass-through
- ✅ Warnings for unsupported features: $ref, anyOf, oneOf, allOf, enum
- ✅ Error on non-object root type
- ✅ Schema ID from $id, title, or fallback
- ✅ CLI auto-detection: `--schema` accepts both formats transparently
- ✅ 23 unit tests including OpenClaw llm-task compatibility test
- ✅ All 65 existing tests remain green (no breaking changes)
- ✅ English identifiers throughout (function names adapted from AP spec: konvertiere → convert, ist → is)

## Next Action
→ Continue with Phase 1 Translation tasks
