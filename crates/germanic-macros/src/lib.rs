//! # GERMANIC Macros
//!
//! Proc-Macro Crate für das GERMANIC Schema-System.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    MACRO-VERARBEITUNGSPIPELINE                  │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │   Rust-Quellcode                                                │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │ #[derive(GermanicSchema)]               │                   │
//! │   │ #[germanic(schema_id = "...", ...)]     │                   │
//! │   │ pub struct PraxisSchema { ... }         │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                      │                                          │
//! │                      ▼                                          │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │           syn::parse()                  │                   │
//! │   │   Token-Stream → DeriveInput (AST)      │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                      │                                          │
//! │                      ▼                                          │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │       darling::FromDeriveInput          │                   │
//! │   │   AST → SchemaOpts (typisierte Daten)   │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                      │                                          │
//! │                      ▼                                          │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │           quote::quote!()               │                   │
//! │   │   Typisierte Daten → Rust-Code          │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                      │                                          │
//! │                      ▼                                          │
//! │   Generierter Rust-Code                                         │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │ impl GermanicSerialize for PraxisSchema │                   │
//! │   │ impl SchemaMetadata for PraxisSchema    │                   │
//! │   │ impl Validation for PraxisSchema        │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Verwendung
//!
//! ```rust,ignore
//! use germanic_macros::GermanicSchema;
//!
//! #[derive(GermanicSchema)]
//! #[germanic(schema_id = "de.gesundheit.praxis.v1")]
//! pub struct PraxisSchema {
//!     #[germanic(required)]
//!     pub name: String,
//!     
//!     pub telefon: Option<String>,
//! }
//! ```

// Proc-Macro Crates dürfen KEINE anderen Items außer Macros exportieren.
// Daher: private Module für die Implementierung.
mod schema;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// # `#[derive(GermanicSchema)]`
///
/// Erzeugt automatisch die Implementierungen für GERMANIC-Schemas.
///
/// ## Attribute auf Struct-Ebene
///
/// | Attribut | Typ | Beschreibung |
/// |----------|-----|--------------|
/// | `schema_id` | String | Eindeutige Schema-ID (z.B. `"de.gesundheit.praxis.v1"`) |
/// | `flatbuffer` | String | Pfad zum FlatBuffer-Typ (z.B. `"de::praxis::Praxis"`) |
///
/// ## Attribute auf Feld-Ebene
///
/// | Attribut | Typ | Beschreibung |
/// |----------|-----|--------------|
/// | `required` | Flag | Feld darf nicht `None`/leer sein |
/// | `default` | Wert | Standardwert wenn nicht angegeben |
///
/// ## Generierte Traits
///
/// 1. **`GermanicSerialize`**: Serialisierung in FlatBuffer-Bytes
/// 2. **`SchemaMetadata`**: Schema-ID und Version
/// 3. **`Validate`**: Validierung der Pflichtfelder
///
/// ## Beispiel
///
/// ```rust,ignore
/// #[derive(GermanicSchema, Deserialize)]
/// #[germanic(
///     schema_id = "de.gesundheit.praxis.v1",
///     flatbuffer = "de::praxis::Praxis"
/// )]
/// pub struct PraxisSchema {
///     #[germanic(required)]
///     pub name: String,
///     
///     #[germanic(required)]
///     pub bezeichnung: String,
///     
///     pub praxisname: Option<String>,
///     
///     #[germanic(default = false)]
///     pub privatpatienten: bool,
/// }
/// ```
#[proc_macro_derive(GermanicSchema, attributes(germanic))]
pub fn derive_germanic_schema(eingabe: TokenStream) -> TokenStream {
    // 1. Parse den Token-Stream in einen AST
    //    DeriveInput enthält: Name, Generics, Attribute, Data (Struct/Enum)
    let ast = parse_macro_input!(eingabe as DeriveInput);

    // 2. Delegiere an die Implementierung im schema-Modul
    //    Bei Fehler: Compiler-Fehler mit aussagekräftiger Meldung
    schema::implementiere_germanic_schema(ast)
        .unwrap_or_else(|fehler| fehler.write_errors().into())
}
