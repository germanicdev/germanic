//! # Schema Traits
//!
//! Defines the contracts (traits) that the macro implements.
//!
//! ## Architecture: Why Traits?
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    TRAIT-BASED ABSTRACTION                                  │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   PROBLEM:                                                                  │
//! │   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
//! │   │  PracticeSchema │  │ RestaurantSchema│  │   HotelSchema   │            │
//! │   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
//! │          ↓                    ↓                    ↓                        │
//! │   How does the compiler treat all these types uniformly?                    │
//! │                                                                             │
//! │   SOLUTION: Common contract (Trait)                                         │
//! │   ┌─────────────────────────────────────────────────────────────┐           │
//! │   │                  trait Validate                             │           │
//! │   │   fn validate(&self) -> Result<(), ValidationError>         │           │
//! │   └─────────────────────────────────────────────────────────────┘           │
//! │          ↑                    ↑                    ↑                        │
//! │   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
//! │   │  PracticeSchema │  │ RestaurantSchema│  │   HotelSchema   │            │
//! │   │ impl Validate   │  │ impl Validate   │  │ impl Validate   │            │
//! │   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
//! │                                                                             │
//! │   Compiler can now work with `dyn Validate` or generics                     │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use crate::error::ValidationError;

// ============================================================================
// SCHEMA METADATA
// ============================================================================

/// Trait for schema metadata.
///
/// Automatically implemented by the `#[derive(GermanicSchema)]` macro.
///
/// ## Usage
///
/// ```rust,ignore
/// use germanic::schema::SchemaMetadata;
///
/// let practice = PracticeSchema { /* ... */ };
/// println!("Schema-ID: {}", practice.schema_id());  // "de.gesundheit.praxis.v1"
/// ```
///
/// ## Architectural Significance
///
/// The schema ID is written to the .grm header and enables:
/// - AI systems can identify the schema
/// - Versioning for backward compatibility
/// - Registry lookup for schema definitions
pub trait SchemaMetadata {
    /// The unique schema ID.
    ///
    /// Format: `"{namespace}.{domain}.{name}.v{version}"`
    /// Example: `"de.gesundheit.praxis.v1"`
    fn schema_id(&self) -> &'static str;

    /// The schema version (1-255).
    ///
    /// Used for migration logic.
    fn schema_version(&self) -> u8;
}

// ============================================================================
// VALIDATION
// ============================================================================

/// Trait for schema validation.
///
/// Checks if all required fields (`#[germanic(required)]`) are filled.
///
/// ## Example
///
/// ```rust,ignore
/// use germanic::schema::Validate;
///
/// let practice = PracticeSchema {
///     name: "".to_string(),  // EMPTY! → Error
///     bezeichnung: "Heilpraktiker".to_string(),
///     // ...
/// };
///
/// match practice.validate() {
///     Ok(()) => println!("All good"),
///     Err(e) => eprintln!("Validation failed: {}", e),
/// }
/// ```
///
/// ## Architectural Significance
///
/// Validation happens **before** FlatBuffer serialization.
/// This guarantees:
/// - Early failure (fail fast)
/// - No corrupt .grm files
/// - Meaningful error messages for the user
pub trait Validate {
    /// Validates the schema.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all required fields are filled
    /// - `Err(ValidationError)` with list of missing fields
    fn validate(&self) -> Result<(), ValidationError>;
}

// ============================================================================
// SERIALIZATION (Placeholder for later)
// ============================================================================

/// Trait for FlatBuffer serialization.
///
/// **Not yet implemented** – coming in Phase 3 of macro development.
///
/// ## Planned Signature
///
/// ```rust,ignore
/// pub trait GermanicSerialize {
///     /// Serializes the schema into FlatBuffer bytes.
///     fn serialize(&self, builder: &mut FlatBufferBuilder) -> WIPOffset<UnionWIPOffset>;
/// }
/// ```
pub trait GermanicSerialize {
    /// Serializes the schema into a byte vector.
    fn to_bytes(&self) -> Vec<u8>;
}

// ============================================================================
// COMPOSITION TRAIT
// ============================================================================

/// Marker trait for complete GERMANIC schemas.
///
/// A type implements `GermanicSchemaComplete` if it implements all
/// necessary traits.
///
/// ## Automatic Implementation
///
/// ```rust,ignore
/// // Automatically for any type that implements all traits:
/// impl<T> GermanicSchemaComplete for T
/// where
///     T: SchemaMetadata + Validate + GermanicSerialize
/// {}
/// ```
pub trait GermanicSchemaComplete: SchemaMetadata + Validate {}

// Blanket implementation: Any type that has all traits is automatically complete
impl<T> GermanicSchemaComplete for T where T: SchemaMetadata + Validate {}
