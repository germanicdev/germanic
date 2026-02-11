//! # GERMANIC
//!
//! Maschinenlesbare Schemas für Websites.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                          GERMANIC ARCHITEKTUR                               │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │    ┌─────────────┐      ┌─────────────┐      ┌─────────────┐               │
//! │    │   Schema    │      │   Compiler  │      │    .grm     │               │
//! │    │ (Rust + FB) │ ──→  │  (validate  │ ──→  │   Datei     │               │
//! │    │             │      │  + compile) │      │             │               │
//! │    └─────────────┘      └─────────────┘      └─────────────┘               │
//! │          │                    │                    │                       │
//! │          ▼                    ▼                    ▼                       │
//! │    ┌─────────────┐      ┌─────────────┐      ┌─────────────┐               │
//! │    │ JSON-Input  │      │ Validierung │      │  Website    │               │
//! │    │ (von Plugin)│      │ + Signatur  │      │ /data.grm   │               │
//! │    └─────────────┘      └─────────────┘      └─────────────┘               │
//! │                                                                             │
//! │    DATENFLUSS: JSON → Rust Struct → FlatBuffer → .grm mit Header           │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Verwendung
//!
//! ```rust,ignore
//! use germanic::GermanicSchema;
//! use serde::Deserialize;
//!
//! #[derive(GermanicSchema, Deserialize)]
//! #[germanic(schema_id = "de.gesundheit.praxis.v1")]
//! pub struct PraxisSchema {
//!     #[germanic(required)]
//!     pub name: String,
//!     pub telefon: Option<String>,
//! }
//!
//! fn main() {
//!     let json = r#"{"name": "Dr. Müller", "telefon": "+49 123 456"}"#;
//!     let praxis: PraxisSchema = serde_json::from_str(json).unwrap();
//!
//!     // Validierung
//!     use germanic::schema::Validieren;
//!     praxis.validiere().expect("Validierung fehlgeschlagen");
//! }
//! ```
extern crate self as germanic;
// ============================================================================
// RE-EXPORTS
// ============================================================================

/// Re-export des Derive-Macros für einfache Verwendung.
/// Ermöglicht: `use germanic::GermanicSchema;`
pub use germanic_macros::GermanicSchema;

// ============================================================================
// MODULE
// ============================================================================

/// Generierte FlatBuffer-Bindings.
///
/// Enthält die von `flatc` generierten Typen:
/// - `generated::praxis::de::gesundheit::{Praxis, Adresse}`
/// - `generated::meta::germanic::common::{GermanicMeta, Signatur, Hinweis}`
pub mod generated;

/// Schema-Definitionen (Rust-Structs mit Macro).
///
/// Enthält die manuell definierten Schemas:
/// - `schemas::praxis::{PraxisSchema, AdresseSchema}`
pub mod schemas;

/// Schema-Traits für Metadaten und Validierung.
pub mod schema;

/// Fehlertypen.
pub mod error;

/// Header und .grm Format.
pub mod types;

/// Kompilierung von JSON zu .grm.
pub mod compiler;

/// Validierung von JSON gegen Schema.
pub mod validator;

// ============================================================================
// PRELUDE
// ============================================================================

/// Häufig verwendete Items für einfachen Import.
///
/// ```rust,ignore
/// use germanic::prelude::*;
/// ```
pub mod prelude {
    pub use crate::GermanicSchema;
    pub use crate::error::{GermanicError, ValidationError};
    pub use crate::schema::{SchemaMetadata, Validate};
    pub use crate::schemas::{AdresseSchema, PraxisSchema};
}
