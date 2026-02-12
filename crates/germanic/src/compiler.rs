//! # JSON → .grm Compiler
//!
//! Compiles JSON data into the binary .grm format.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    COMPILATION PIPELINE                                     │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   INPUT                         PROCESSING                      OUTPUT      │
//! │   ┌─────────┐                   ┌─────────────┐                ┌─────────┐  │
//! │   │ praxis  │                   │             │                │         │  │
//! │   │  .json  │ ──→ Parse ──→     │ PracticeSchema ──→ Serialize │ .grm    │  │
//! │   │         │                   │             │                │         │  │
//! │   └─────────┘                   └─────────────┘                └─────────┘  │
//! │        │                              │                             │       │
//! │        ▼                              ▼                             ▼       │
//! │   serde_json::from_str          1. validate()               GrmHeader +     │
//! │                                 2. to_bytes()               FlatBuffer      │
//! │                                                                             │
//! │   ERROR POINTS:                                                             │
//! │   1. Invalid JSON syntax         → JsonError                                │
//! │   2. Wrong schema structure      → DeserializeError                         │
//! │   3. Missing required fields     → ValidationError                          │
//! │   4. IO error when writing       → IoError                                  │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

use crate::error::{GermanicError, GermanicResult};
use crate::schema::{GermanicSerialize, SchemaMetadata, Validate};
use crate::types::GrmHeader;
use serde::de::DeserializeOwned;
use std::path::Path;

// ============================================================================
// COMPILATION
// ============================================================================

/// Compiles a schema object to .grm bytes.
///
/// ## Pipeline
///
/// ```text
/// Schema ──► validate() ──► to_bytes() ──► Header + Payload
/// ```
///
/// ## Architectural Guiding Questions:
///
/// 1. **Who validates?** The compiler, before bytes are written.
/// 2. **What happens on errors?** Fail-fast with meaningful message.
/// 3. **Who owns the data?** Immutable borrow (`&schema`).
///
/// ## Example
///
/// ```rust,ignore
/// use germanic::compiler::compile;
/// use germanic::schemas::PracticeSchema;
///
/// let practice = PracticeSchema {
///     name: "Dr. Anna Schmidt".to_string(),
///     bezeichnung: "Zahnärztin".to_string(),
///     // ...
/// };
///
/// let bytes = compile(&practice)?;
/// std::fs::write("practice.grm", bytes)?;
/// ```
pub fn compile<S>(schema: &S) -> GermanicResult<Vec<u8>>
where
    S: SchemaMetadata + Validate + GermanicSerialize,
{
    // 1. Validate required fields
    schema.validate().map_err(GermanicError::Validation)?;

    // 2. Create header
    let header = GrmHeader::new(schema.schema_id());
    let header_bytes = header.to_bytes();

    // 3. Serialize schema to FlatBuffer
    let payload_bytes = schema.to_bytes();

    // 4. Combine header + payload
    let mut output = Vec::with_capacity(header_bytes.len() + payload_bytes.len());
    output.extend_from_slice(&header_bytes);
    output.extend_from_slice(&payload_bytes);

    Ok(output)
}

/// Compiles JSON string to .grm bytes.
///
/// This is the main function for the Concierge workflow:
/// 1. Plugin exports JSON
/// 2. CLI calls this function
/// 3. .grm is generated
///
/// ## Example
///
/// ```rust,ignore
/// use germanic::compiler::compile_json;
/// use germanic::schemas::PracticeSchema;
///
/// let json = std::fs::read_to_string("practice.json")?;
/// let bytes = compile_json::<PracticeSchema>(&json)?;
/// std::fs::write("practice.grm", bytes)?;
/// ```
pub fn compile_json<S>(json: &str) -> GermanicResult<Vec<u8>>
where
    S: DeserializeOwned + SchemaMetadata + Validate + GermanicSerialize,
{
    // 1. Parse JSON to Rust struct
    let schema: S = serde_json::from_str(json)?;

    // 2. Delegate to compile()
    compile(&schema)
}

/// Compiles a JSON file to .grm bytes.
///
/// ## Example
///
/// ```rust,ignore
/// use germanic::compiler::compile_file;
/// use germanic::schemas::PracticeSchema;
///
/// let bytes = compile_file::<PracticeSchema>(Path::new("practice.json"))?;
/// ```
pub fn compile_file<S>(path: &Path) -> GermanicResult<Vec<u8>>
where
    S: DeserializeOwned + SchemaMetadata + Validate + GermanicSerialize,
{
    let json = std::fs::read_to_string(path)?;
    compile_json::<S>(&json)
}

/// Writes .grm bytes to a file.
///
/// ## Example
///
/// ```rust,ignore
/// let bytes = compile(&practice)?;
/// write_grm(&bytes, Path::new("practice.grm"))?;
/// ```
pub fn write_grm(data: &[u8], path: &Path) -> GermanicResult<()> {
    std::fs::write(path, data)?;
    Ok(())
}

// ============================================================================
// SCHEMA REGISTRY (for CLI)
// ============================================================================

/// Known schema types for the CLI.
///
/// The CLI command `germanic compile --schema practice` needs
/// a mapping from string names to concrete types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaType {
    /// Practice schema for healthcare practitioners
    Practice,
}

impl SchemaType {
    /// Parses a schema name from a string.
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "praxis" | "practice" => Some(Self::Practice),
            _ => None,
        }
    }

    /// Returns the schema name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Practice => "practice",
        }
    }

    /// Returns the schema ID.
    pub fn schema_id(&self) -> &'static str {
        match self {
            Self::Practice => "de.gesundheit.praxis.v1",
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{AdresseSchema, PraxisSchema};

    #[test]
    fn test_schema_type_parsing() {
        assert_eq!(SchemaType::parse("praxis"), Some(SchemaType::Practice));
        assert_eq!(SchemaType::parse("practice"), Some(SchemaType::Practice));
        assert_eq!(SchemaType::parse("PRAXIS"), Some(SchemaType::Practice));
        assert_eq!(SchemaType::parse("unknown"), None);
    }

    #[test]
    fn test_compile_practice() {
        let practice = PraxisSchema {
            name: "Test".to_string(),
            bezeichnung: "Arzt".to_string(),
            adresse: AdresseSchema {
                strasse: "Teststr.".to_string(),
                hausnummer: None,
                plz: "12345".to_string(),
                ort: "Berlin".to_string(),
                land: "DE".to_string(),
            },
            ..Default::default()
        };

        let bytes = compile(&practice).expect("Compilation should succeed");

        // Check header (magic bytes)
        assert_eq!(&bytes[0..3], b"GRM");

        // Check schema-ID in header
        let schema_id_len = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
        let schema_id = std::str::from_utf8(&bytes[6..6 + schema_id_len]).unwrap();
        assert_eq!(schema_id, "de.gesundheit.praxis.v1");
    }

    #[test]
    fn test_compile_json_practice() {
        let json = r#"{
            "name": "Dr. Müller",
            "bezeichnung": "Arzt",
            "adresse": {
                "strasse": "Hauptstraße",
                "plz": "12345",
                "ort": "Berlin"
            }
        }"#;

        let bytes = compile_json::<PraxisSchema>(json).expect("Compilation should succeed");

        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..3], b"GRM");
    }

    #[test]
    fn test_compile_validation_error() {
        let practice = PraxisSchema::default(); // All required fields empty

        let result = compile(&practice);

        assert!(result.is_err());
        assert!(matches!(result, Err(GermanicError::Validation(_))));
    }
}
