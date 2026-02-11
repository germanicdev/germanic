//! # Error Types
//!
//! Defines all errors that can occur in GERMANIC.
//!
//! ## Architecture: Errors as Types
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                    ERROR HIERARCHY                                          │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │                      GermanicError                                          │
//! │                           │                                                 │
//! │       ┌───────────────────┼───────────────────┐                             │
//! │       │                   │                   │                             │
//! │       ▼                   ▼                   ▼                             │
//! │  Validation        Serialization       Compilation                          │
//! │       │                   │                   │                             │
//! │       ▼                   ▼                   ▼                             │
//! │  RequiredFieldsMissing  FlatBufferError  FileNotFound                       │
//! │  TypeError              SignatureError   SchemaError                        │
//! │                                                                             │
//! │  PRINCIPLE: Each error has its own type with specific data                  │
//! │             No string-based error messages!                                 │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Why `thiserror`?
//!
//! `thiserror` automatically generates:
//! - `std::error::Error` implementation
//! - `Display` implementation (for error messages)
//! - `From` implementations (for `?` operator)

use thiserror::Error;

// ============================================================================
// MAIN ERROR TYPE
// ============================================================================

/// Main error type for all GERMANIC operations.
///
/// ## Usage
///
/// ```rust,ignore
/// use germanic::error::GermanicError;
///
/// fn compile(json: &str) -> Result<Vec<u8>, GermanicError> {
///     let practice: PracticeSchema = serde_json::from_str(json)?;  // → JsonError
///     practice.validate()?;  // → Validation
///     // ...
/// }
/// ```
#[derive(Error, Debug)]
pub enum GermanicError {
    /// Validation error (required fields, types)
    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationError),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Filesystem error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Schema not found
    #[error("Unknown schema: {0}")]
    UnknownSchema(String),

    /// General error with message
    #[error("{0}")]
    General(String),
}

// ============================================================================
// VALIDATION ERRORS
// ============================================================================

/// Error during schema validation.
///
/// ## Example
///
/// ```rust,ignore
/// match practice.validate() {
///     Err(ValidationError::RequiredFieldsMissing(fields)) => {
///         eprintln!("Missing fields: {:?}", fields);
///         // → "Missing fields: ["name", "adresse"]"
///     }
///     _ => {}
/// }
/// ```
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    /// Required fields are empty or missing.
    #[error("Required fields missing: {}", field_list(.0))]
    RequiredFieldsMissing(Vec<String>),

    /// Field value has wrong type.
    #[error("Type error in field '{field}': expected {expected}, found {found}")]
    TypeError {
        field: String,
        expected: String,
        found: String,
    },

    /// Field value violates constraints.
    #[error("Constraint violation in field '{field}': {message}")]
    ConstraintViolation { field: String, message: String },
}

/// Helper function: formats field list as comma-separated string.
fn field_list(fields: &[String]) -> String {
    if fields.is_empty() {
        "(none)".to_string()
    } else {
        fields.join(", ")
    }
}

// ============================================================================
// COMPILATION ERRORS
// ============================================================================

/// Error during compilation to .grm.
#[derive(Error, Debug)]
pub enum CompilationError {
    /// Input file not found.
    #[error("Input file not found: {path}")]
    FileNotFound { path: String },

    /// Output could not be written.
    #[error("Output error: {message}")]
    OutputError { message: String },

    /// FlatBuffer serialization failed.
    #[error("Serialization failed: {message}")]
    SerializationError { message: String },
}

// ============================================================================
// RESULT TYPE ALIAS
// ============================================================================

/// Convenient alias for GERMANIC operations.
///
/// ```rust,ignore
/// fn my_function() -> GermanicResult<Vec<u8>> {
///     // ...
/// }
/// ```
pub type GermanicResult<T> = Result<T, GermanicError>;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_fields_missing_display() {
        let error = ValidationError::RequiredFieldsMissing(vec!["name".into(), "adresse".into()]);

        assert_eq!(error.to_string(), "Required fields missing: name, adresse");
    }

    #[test]
    fn test_empty_required_fields() {
        let error = ValidationError::RequiredFieldsMissing(vec![]);

        assert_eq!(error.to_string(), "Required fields missing: (none)");
    }

    #[test]
    fn test_error_conversion() {
        let validation_error = ValidationError::RequiredFieldsMissing(vec!["name".into()]);

        let germanic_error: GermanicError = validation_error.into();

        assert!(matches!(germanic_error, GermanicError::Validation(_)));
    }
}
