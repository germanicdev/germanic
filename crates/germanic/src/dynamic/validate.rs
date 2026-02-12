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

/// Recursively validates fields, collecting all violations with path prefixes.
///
/// Validation chain per field (order matters!):
/// 1. Field present? → if missing and required → error
/// 2. Value == null? → if null and required → error
/// 3. Type correct?  → if mismatch → error
/// 4. Empty check    → "" or [] for required → error
/// 5. Nested table?  → recurse
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
            // Check 1: Field missing
            None => {
                if def.required {
                    errors.push(format!("{}: required field missing", path));
                }
            }
            Some(value) => {
                // Check 2: Null for required field
                if value.is_null() {
                    if def.required {
                        errors.push(format!("{}: null value for required field", path));
                    }
                    continue;
                }

                // Check 3: Type mismatch
                if !type_matches(&def.field_type, value) {
                    errors.push(format!(
                        "{}: expected {}, found {}",
                        path,
                        field_type_name(&def.field_type),
                        value_type_name(value)
                    ));
                    continue; // No empty-check on wrong type
                }

                // Check 4: Empty check for required fields
                if def.required {
                    match (&def.field_type, value) {
                        (FieldType::String, serde_json::Value::String(s)) if s.is_empty() => {
                            errors.push(format!("{}: required field is empty string", path));
                        }
                        (FieldType::StringArray, serde_json::Value::Array(a)) if a.is_empty() => {
                            errors.push(format!("{}: required array is empty", path));
                        }
                        _ => {}
                    }
                }

                // Check 5: Recurse into nested tables
                if def.field_type == FieldType::Table {
                    if let Some(nested_fields) = &def.fields {
                        if let Some(nested_obj) = value.as_object() {
                            validate_fields(nested_fields, nested_obj, &path, errors);
                        } else if def.required {
                            errors.push(format!(
                                "{}: expected table, found {}",
                                path,
                                value_type_name(value)
                            ));
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

/// Checks if a JSON value matches the expected schema field type.
///
/// This is the type contract: the schema says "bool", the JSON must deliver bool.
/// Null is handled separately (before this check), so null returns true here.
fn type_matches(expected: &FieldType, value: &serde_json::Value) -> bool {
    match (expected, value) {
        // Null handled separately — not a type mismatch
        (_, serde_json::Value::Null) => true,

        // Exact type matches
        (FieldType::String, serde_json::Value::String(_)) => true,
        (FieldType::Bool, serde_json::Value::Bool(_)) => true,
        (FieldType::Int, serde_json::Value::Number(n)) => n.is_i64(),
        (FieldType::Float, serde_json::Value::Number(n)) => n.is_f64(),

        // Arrays — check container type (element check is future work)
        (FieldType::StringArray, serde_json::Value::Array(_)) => true,
        (FieldType::IntArray, serde_json::Value::Array(_)) => true,

        // Tables
        (FieldType::Table, serde_json::Value::Object(_)) => true,

        // Everything else: mismatch
        _ => false,
    }
}

/// Returns a human-readable name for a FieldType.
fn field_type_name(ft: &FieldType) -> &'static str {
    match ft {
        FieldType::String => "string",
        FieldType::Bool => "bool",
        FieldType::Int => "int",
        FieldType::Float => "float",
        FieldType::StringArray => "[string]",
        FieldType::IntArray => "[int]",
        FieldType::Table => "table",
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
        if let ValidationError::RequiredFieldsMissing(violations) = err {
            assert!(violations.iter().any(|v| v.starts_with("name:")));
        }
    }

    #[test]
    fn test_empty_string_required() {
        let schema = simple_schema();
        let data: serde_json::Value = serde_json::json!({ "name": "" });
        let err = validate_against_schema(&schema, &data).unwrap_err();
        if let ValidationError::RequiredFieldsMissing(violations) = err {
            assert!(violations.iter().any(|v| v.starts_with("name:")));
        }
    }

    #[test]
    fn test_optional_missing_ok() {
        let schema = simple_schema();
        let data: serde_json::Value = serde_json::json!({ "name": "Bistro" });
        assert!(validate_against_schema(&schema, &data).is_ok());
    }
}
