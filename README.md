# GERMANIC

Maschinenlesbare Schemas für Websites – damit KIs Praxis-Daten korrekt verstehen.

## Quick Start

```bash
# 1. Entpacken
tar -xzvf germanic-workspace.tar.gz
cd germanic

# 2. FlatBuffers-Compiler installieren (falls nicht vorhanden)
brew install flatbuffers  # macOS

# 3. Kompilieren
cargo build

# 4. Praxis kompilieren (nach Macro-Implementierung)
./target/release/germanic compile \
    --schema praxis \
    --input examples/dr-sonnenschein.json
```

## Struktur

```
germanic/
├── Cargo.toml                 # Workspace
├── LICENSE                    # Proprietär
│
├── crates/
│   ├── germanic/              # CLI + Library
│   │   ├── Cargo.toml
│   │   ├── build.rs           # FlatBuffer-Kompilierung + Namespace-Fix
│   │   └── src/
│   │       ├── lib.rs         # Public API
│   │       ├── main.rs        # CLI
│   │       ├── generated.rs   # Re-export FlatBuffer-Bindings
│   │       ├── schema.rs      # Traits
│   │       ├── fehler.rs      # Fehlertypen
│   │       ├── types.rs       # .grm Format
│   │       ├── compiler.rs    # JSON → .grm
│   │       └── validator.rs   # Validierung
│   │
│   └── germanic-macros/       # Proc-Macro Crate
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs         # #[derive(GermanicSchema)]
│           └── schema.rs      # darling-basiertes Parsing
│
├── schemas/
│   ├── common/meta.fbs        # GermanicMeta, Signatur, Hinweis
│   └── de/praxis.fbs          # Praxis, Adresse
│
└── examples/
    ├── dr-sonnenschein.json         # Test-Daten (fictional)
    └── heilpraxis-waldberg.json     # Test-Daten (fictional)
```

## FlatBuffers Namespace-Bug (#5275)

Das build.rs fixt automatisch den seit 2019 offenen Bug:

```
flatc generiert:    super::super::germanic::common::GermanicMeta
Wir brauchen:       crate::generated::meta::germanic::common::GermanicMeta
```

Der Fix ist ein simpler String-Replace nach der Codegenerierung.

## Nächste Schritte

```
[x] A1: build.rs mit Namespace-Fix
[x] A2: generated.rs Modul
[ ] A3: Kompilierung testen (cargo build)
[ ] B1: Macro-Kompilierung testen
[ ] B2: Validieren Trait verifizieren
[ ] C1: PraxisSchema mit Macro
[ ] D2: CLI-Integration
[ ] D4: End-to-End Test
```

---

*Proprietäre Software – Alle Rechte vorbehalten*
