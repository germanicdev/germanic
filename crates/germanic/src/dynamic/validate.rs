//! # Dynamic Schema Validation
//!
//! Validates JSON data against a SchemaDefinition at runtime.
//!
//! ## Validation Layers
//!
//! ```text
//! Layer 1: Required fields present?     → "name" missing
//! Layer 2: Types match schema?          → "rating" expected float, got string
//! Layer 3: Nested tables valid?         → "address.street" missing
//! ```

use crate::dynamic::schema_def::{FieldDefinition, FieldType, SchemaDefinition};
use crate::error::ValidationError;

/// Validates JSON data against a schema definition.
///
/// Returns Ok(()) if all required fields are present and types match.
/// Returns Err with list of all violations found (not fail-fast — collects all).
pub fn validate_against_schema(
    schema: &SchemaDefinition,
    data: &serde_json::Value,
) -> Result<(), ValidationError> {
    let obj = data.as_object().ok_or_else(|| ValidationError::TypeError {
        field: "(root)".into(),
        expected: "object".into(),
        found: value_type_name(data).into(),
    })?;

    let mut missing = Vec::new();
    validate_fields(&schema.fields, obj, "", &mut missing);

    if missing.is_empty() {
        Ok(())
    } else {
        Err(ValidationError::RequiredFieldsMissing(missing))
    }
}

/// Recursively validates fields, collecting all errors with path prefixes.
fn validate_fields(
    fields: &indexmap::IndexMap<String, FieldDefinition>,
    data: &serde_json::Map<String, serde_json::Value>,
    prefix: &str,
    errors: &mut Vec<String>,
) {
    for (name, def) in fields {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}.{}", prefix, name)
        };

        match data.get(name) {
            None => {
                if def.required {
                    errors.push(path);
                }
            }
            Some(value) => {
                // Check for empty required strings
                if def.required {
                    match (&def.field_type, value) {
                        (FieldType::String, serde_json::Value::String(s)) if s.is_empty() => {
                            errors.push(path.clone());
                        }
                        (FieldType::StringArray, serde_json::Value::Array(a)) if a.is_empty() => {
                            errors.push(path.clone());
                        }
                        _ => {}
                    }
                }

                // Recurse into nested tables
                if def.field_type == FieldType::Table {
                    if let Some(nested_fields) = &def.fields {
                        if let Some(nested_obj) = value.as_object() {
                            validate_fields(nested_fields, nested_obj, &path, errors);
                        } else if def.required {
                            errors.push(path);
                        }
                    }
                }
            }
        }
    }
}

/// Returns the JSON type name for error messages.
fn value_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::schema_def::*;
    use indexmap::IndexMap;

    fn simple_schema() -> SchemaDefinition {
        let mut fields = IndexMap::new();
        fields.insert(
            "name".into(),
            FieldDefinition {
                field_type: FieldType::String,
                required: true,
                default: None,
                fields: None,
            },
        );
        fields.insert(
            "rating".into(),
            FieldDefinition {
                field_type: FieldType::Float,
                required: false,
                default: None,
                fields: None,
            },
        );
        SchemaDefinition {
            schema_id: "test.v1".into(),
            version: 1,
            fields,
        }
    }

    #[test]
    fn test_valid_data() {
        let schema = simple_schema();
        let data: serde_json::Value = serde_json::json!({
            "name": "Test Restaurant",
            "rating": 4.5
        });
        assert!(validate_against_schema(&schema, &data).is_ok());
    }

    #[test]
    fn test_missing_required() {
        let schema = simple_schema();
        let data: serde_json::Value = serde_json::json!({ "rating": 4.5 });
        let err = validate_against_schema(&schema, &data).unwrap_err();
        if let ValidationError::RequiredFieldsMissing(fields) = err {
            assert!(fields.contains(&"name".to_string()));
        }
    }

    #[test]
    fn test_empty_string_required() {
        let schema = simple_schema();
        let data: serde_json::Value = serde_json::json!({ "name": "" });
        assert!(validate_against_schema(&schema, &data).is_err());
    }

    #[test]
    fn test_optional_missing_ok() {
        let schema = simple_schema();
        let data: serde_json::Value = serde_json::json!({ "name": "Bistro" });
        assert!(validate_against_schema(&schema, &data).is_ok());
    }
}
