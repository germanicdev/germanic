//! # GERMANIC Macros
//!
//! Proc-macro crate for the GERMANIC schema system.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    MACRO PROCESSING PIPELINE                    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │   Rust Source Code                                              │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │ #[derive(GermanicSchema)]               │                   │
//! │   │ #[germanic(schema_id = "...", ...)]     │                   │
//! │   │ pub struct PracticeSchema { ... }       │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                      │                                          │
//! │                      ▼                                          │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │           syn::parse()                  │                   │
//! │   │   Token Stream → DeriveInput (AST)      │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                      │                                          │
//! │                      ▼                                          │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │       darling::FromDeriveInput          │                   │
//! │   │   AST → SchemaOpts (typed data)         │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                      │                                          │
//! │                      ▼                                          │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │           quote::quote!()               │                   │
//! │   │   Typed Data → Rust Code                │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                      │                                          │
//! │                      ▼                                          │
//! │   Generated Rust Code                                           │
//! │   ┌─────────────────────────────────────────┐                   │
//! │   │ impl GermanicSerialize for PracticeSchema│                  │
//! │   │ impl SchemaMetadata for PracticeSchema  │                   │
//! │   │ impl Validate for PracticeSchema        │                   │
//! │   └─────────────────────────────────────────┘                   │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use germanic_macros::GermanicSchema;
//!
//! #[derive(GermanicSchema)]
//! #[germanic(schema_id = "de.gesundheit.praxis.v1")]
//! pub struct PracticeSchema {
//!     #[germanic(required)]
//!     pub name: String,
//!
//!     pub telefon: Option<String>,
//! }
//! ```

// Proc-macro crates may ONLY export macros, no other items.
// Therefore: private modules for implementation.
mod schema;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// # `#[derive(GermanicSchema)]`
///
/// Automatically generates implementations for GERMANIC schemas.
///
/// ## Struct-level Attributes
///
/// | Attribute | Type | Description |
/// |----------|------|-------------|
/// | `schema_id` | String | Unique schema ID (e.g. `"de.gesundheit.praxis.v1"`) |
/// | `flatbuffer` | String | Path to FlatBuffer type (e.g. `"de::praxis::Praxis"`) |
///
/// ## Field-level Attributes
///
/// | Attribute | Type | Description |
/// |----------|------|-------------|
/// | `required` | Flag | Field must not be `None`/empty |
/// | `default` | Value | Default value if not specified |
///
/// ## Generated Traits
///
/// 1. **`GermanicSerialize`**: Serialization to FlatBuffer bytes
/// 2. **`SchemaMetadata`**: Schema ID and version
/// 3. **`Validate`**: Validation of required fields
///
/// ## Example
///
/// ```rust,ignore
/// #[derive(GermanicSchema, Deserialize)]
/// #[germanic(
///     schema_id = "de.gesundheit.praxis.v1",
///     flatbuffer = "de::praxis::Praxis"
/// )]
/// pub struct PracticeSchema {
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
pub fn derive_germanic_schema(input: TokenStream) -> TokenStream {
    // 1. Parse the token stream into an AST
    //    DeriveInput contains: Name, Generics, Attributes, Data (Struct/Enum)
    let ast = parse_macro_input!(input as DeriveInput);

    // 2. Delegate to implementation in schema module
    //    On error: Compiler error with meaningful message
    schema::implement_germanic_schema(ast).unwrap_or_else(|error| error.write_errors().into())
}
