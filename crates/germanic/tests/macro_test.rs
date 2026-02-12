//! Integration tests for the GermanicSchema macro
//!
//! Tests:
//! - Validate trait (required fields)
//! - Default trait (default values)
//! - SchemaMetadata trait (schema_id)

use germanic::GermanicSchema;
use germanic::schema::{SchemaMetadata, Validate};

// ============================================================================
// TEST 1: Validation of required fields
// ============================================================================

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.validation.v1")]
pub struct ValidationTestSchema {
    #[germanic(required)]
    pub name: String,

    pub optional: Option<String>,
}

#[test]
fn test_validation_name_empty() {
    let schema = ValidationTestSchema {
        name: "".to_string(),
        optional: None,
    };

    // Empty required string should throw error
    let result = schema.validate();
    assert!(result.is_err());
}

#[test]
fn test_validation_ok() {
    let schema = ValidationTestSchema {
        name: "Test".to_string(),
        optional: None,
    };

    assert!(schema.validate().is_ok());
}

// ============================================================================
// TEST 2: Default Trait
// ============================================================================

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.default.v1")]
pub struct DefaultTestSchema {
    #[germanic(default = "Deutschland")]
    pub land: String,

    #[germanic(default = "true")]
    pub aktiv: bool,

    pub name: String, // No default → String::new()

    pub optional: Option<String>, // No default → None

    pub list: Vec<String>, // No default → Vec::new()
}

#[test]
fn test_default_trait() {
    let schema = DefaultTestSchema::default();

    assert_eq!(schema.land, "Deutschland");
    assert!(schema.aktiv);
    assert_eq!(schema.name, "");
    assert!(schema.optional.is_none());
    assert!(schema.list.is_empty());
}

#[test]
fn test_default_bool_false() {
    #[derive(GermanicSchema)]
    #[germanic(schema_id = "test.bool.v1")]
    pub struct BoolTestSchema {
        #[germanic(default = "false")]
        pub deactivated: bool,

        pub without_default: bool, // → false
    }

    let schema = BoolTestSchema::default();
    assert!(!schema.deactivated);
    assert!(!schema.without_default);
}

// ============================================================================
// TEST 3: SchemaMetadata Trait
// ============================================================================

#[test]
fn test_schema_metadata() {
    let schema = ValidationTestSchema {
        name: "Test".to_string(),
        optional: None,
    };

    assert_eq!(schema.schema_id(), "test.validation.v1");
    assert_eq!(schema.schema_version(), 1);
}

// ============================================================================
// TEST 4: Combined validation and default
// ============================================================================

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.combined.v1")]
pub struct CombinedTestSchema {
    #[germanic(required)]
    pub required: String,

    #[germanic(default = "Standard")]
    pub with_default: String,

    #[germanic(required)]
    pub required_vec: Vec<String>,
}

#[test]
fn test_default_does_not_satisfy_required() {
    // Default creates empty strings/vecs which should fail for required fields
    let schema = CombinedTestSchema::default();

    let result = schema.validate();
    assert!(result.is_err());

    // Check which fields are missing
    if let Err(germanic::error::ValidationError::RequiredFieldsMissing(fields)) = result {
        assert!(fields.contains(&"required".to_string()));
        assert!(fields.contains(&"required_vec".to_string()));
        // with_default has a value, should NOT be in error list
        assert!(!fields.contains(&"with_default".to_string()));
    }
}

#[test]
fn test_combined_valid() {
    let schema = CombinedTestSchema {
        required: "Filled".to_string(),
        with_default: "Overridden".to_string(),
        required_vec: vec!["Entry".to_string()],
    };

    assert!(schema.validate().is_ok());
}

// ============================================================================
// TEST 5: Nested Structs
// ============================================================================

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.adresse.v1")]
pub struct AdresseTestSchema {
    #[germanic(required)]
    pub strasse: String,

    #[germanic(required)]
    pub plz: String,

    #[germanic(required)]
    pub ort: String,

    #[germanic(default = "DE")]
    pub land: String,
}

#[derive(GermanicSchema)]
#[germanic(schema_id = "test.praxis.v1")]
pub struct PraxisTestSchema {
    #[germanic(required)]
    pub name: String,

    pub adresse: AdresseTestSchema, // Nested Struct
}

#[test]
fn test_nested_default() {
    let schema = PraxisTestSchema::default();

    assert_eq!(schema.name, "");
    assert_eq!(schema.adresse.strasse, "");
    assert_eq!(schema.adresse.land, "DE");
}

#[test]
fn test_nested_validation_error() {
    let schema = PraxisTestSchema::default();

    let result = schema.validate();
    assert!(result.is_err());

    if let Err(germanic::error::ValidationError::RequiredFieldsMissing(fields)) = result {
        // Main field
        assert!(fields.contains(&"name".to_string()));
        // Nested fields with prefix
        assert!(fields.contains(&"adresse.strasse".to_string()));
        assert!(fields.contains(&"adresse.plz".to_string()));
        assert!(fields.contains(&"adresse.ort".to_string()));
    }
}

#[test]
fn test_nested_validation_ok() {
    let schema = PraxisTestSchema {
        name: "Dr. Müller".to_string(),
        adresse: AdresseTestSchema {
            strasse: "Hauptstraße 1".to_string(),
            plz: "12345".to_string(),
            ort: "Berlin".to_string(),
            land: "DE".to_string(),
        },
    };

    assert!(schema.validate().is_ok());
}

#[test]
fn test_nested_partial_error() {
    // Only the nested struct has errors
    let schema = PraxisTestSchema {
        name: "Dr. Müller".to_string(), // OK
        adresse: AdresseTestSchema {
            strasse: "".to_string(), // ERROR
            plz: "12345".to_string(),
            ort: "Berlin".to_string(),
            land: "DE".to_string(),
        },
    };

    let result = schema.validate();
    assert!(result.is_err());

    if let Err(germanic::error::ValidationError::RequiredFieldsMissing(fields)) = result {
        assert_eq!(fields.len(), 1);
        assert!(fields.contains(&"adresse.strasse".to_string()));
    }
}
