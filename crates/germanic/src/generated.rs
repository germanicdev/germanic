//! # Generierte FlatBuffer-Bindings
//!
//! Dieses Modul inkludiert den von `flatc` generierten Rust-Code.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    FLATC OUTPUT-STRUKTUR                                    │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   flatc generiert EINE Datei pro .fbs mit ALLEN Typen darin:               │
//! │                                                                             │
//! │   meta.fbs (namespace germanic.common)                                      │
//! │       → meta_generated.rs                                                   │
//! │           └── mod germanic { mod common { ... } }                           │
//! │                                                                             │
//! │   praxis.fbs (namespace de.gesundheit)                                      │
//! │       → praxis_generated.rs                                                 │
//! │           └── mod de { mod gesundheit { ... } }                             │
//! │                                                                             │
//! │   VERWENDUNG:                                                               │
//! │   use crate::generated::praxis::de::gesundheit::Praxis;                     │
//! │   use crate::generated::meta::germanic::common::GermanicMeta;               │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Warum `include!` statt `mod`?
//!
//! Die generierten Dateien liegen in `$OUT_DIR`, nicht im `src/` Verzeichnis.
//! Rust's `mod` funktioniert nur für Dateien im selben Verzeichnisbaum.
//! `include!` fügt den Inhalt direkt ein.

//! # Generierte FlatBuffer-Bindings
//!
//! Inkludiert den von `flatc` generierten Rust-Code.
//!
//! ## Modulstruktur (von flatc generiert)
//!
//! ```text
//! meta_generated.rs    → mod germanic { mod meta { Signatur, Meta, ... } }
//! praxis_generated.rs  → mod de { mod gesundheit { Adresse, Praxis } }
//! ```

#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::all)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

// ============================================================================
// META-SCHEMA (aus common/meta.fbs)
// ============================================================================

pub mod meta {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/meta_generated.rs"));
}

// ============================================================================
// PRAXIS-SCHEMA (aus de/praxis.fbs)
// ============================================================================

pub mod praxis {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/praxis_generated.rs"));
}

// ============================================================================
// RE-EXPORTS
// ============================================================================

// Meta-Typen: crate::generated::meta::germanic::common::*
pub use meta::germanic::common::{
    GermanicMeta, GermanicMetaArgs, Hinweis, HinweisArgs, Signatur, SignaturArgs,
};

// Praxis-Typen: crate::generated::praxis::de::gesundheit::*
pub use praxis::de::gesundheit::{Adresse, AdresseArgs, Praxis, PraxisArgs};
