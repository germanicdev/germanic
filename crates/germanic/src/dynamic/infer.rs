//! # Schema Inference
//!
//! Infers a SchemaDefinition from example JSON data.
//!
//! ## Algorithm
//!
//! ```text
//! JSON Value              →  FieldType
//! ─────────────────────────────────────
//! "hello"                 →  String
//! true / false            →  Bool
//! 42 (integer)            →  Int
//! 3.14 (has decimal)      →  Float
//! ["a", "b"]              →  StringArray
//! [1, 2, 3]               →  IntArray
//! { "key": ... }          →  Table (recurse)
//! null                    →  String (fallback)
//! ```
//!
//! All fields default to `required: false`. The user edits
//! the generated .schema.json to mark required fields.

use crate::dynamic::schema_def::{FieldDefinition, FieldType, SchemaDefinition};
use indexmap::IndexMap;

/// Infers a schema definition from example JSON data.
///
/// The schema_id must be provided (cannot be inferred from data).
/// All fields are initially marked as optional — user edits .schema.json to set required.
pub fn infer_schema(data: &serde_json::Value, schema_id: &str) -> Option<SchemaDefinition> {
    let obj = data.as_object()?;

    let fields = infer_fields(obj);

    Some(SchemaDefinition {
        schema_id: schema_id.to_string(),
        version: 1,
        fields,
    })
}

/// Infers field definitions from a JSON object.
fn infer_fields(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> IndexMap<String, FieldDefinition> {
    let mut fields = IndexMap::new();

    for (key, value) in obj {
        let def = infer_field(value);
        fields.insert(key.clone(), def);
    }

    fields
}

/// Infers a single field definition from a JSON value.
fn infer_field(value: &serde_json::Value) -> FieldDefinition {
    match value {
        serde_json::Value::String(_) => FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },

        serde_json::Value::Bool(_) => FieldDefinition {
            field_type: FieldType::Bool,
            required: false,
            default: Some("false".into()),
            fields: None,
        },

        serde_json::Value::Number(n) => {
            let field_type = if n.is_f64() && n.to_string().contains('.') {
                FieldType::Float
            } else {
                FieldType::Int
            };
            FieldDefinition {
                field_type,
                required: false,
                default: None,
                fields: None,
            }
        }

        serde_json::Value::Array(arr) => {
            let field_type = infer_array_type(arr);
            FieldDefinition {
                field_type,
                required: false,
                default: None,
                fields: None,
            }
        }

        serde_json::Value::Object(obj) => {
            let nested = infer_fields(obj);
            FieldDefinition {
                field_type: FieldType::Table,
                required: false,
                default: None,
                fields: Some(nested),
            }
        }

        serde_json::Value::Null => FieldDefinition {
            field_type: FieldType::String,
            required: false,
            default: None,
            fields: None,
        },
    }
}

/// Infers array element type. Defaults to StringArray if empty or mixed.
fn infer_array_type(arr: &[serde_json::Value]) -> FieldType {
    if arr.is_empty() {
        return FieldType::StringArray;
    }

    let first = &arr[0];
    if first.is_number() && arr.iter().all(|v| v.is_number()) {
        FieldType::IntArray
    } else {
        FieldType::StringArray
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_simple() {
        let json: serde_json::Value = serde_json::json!({
            "name": "Test",
            "rating": 4.5,
            "active": true,
            "tags": ["a", "b"]
        });

        let schema = infer_schema(&json, "test.v1").unwrap();
        assert_eq!(schema.fields["name"].field_type, FieldType::String);
        assert_eq!(schema.fields["rating"].field_type, FieldType::Float);
        assert_eq!(schema.fields["active"].field_type, FieldType::Bool);
        assert_eq!(schema.fields["tags"].field_type, FieldType::StringArray);
    }

    #[test]
    fn test_infer_nested() {
        let json: serde_json::Value = serde_json::json!({
            "name": "Test",
            "address": {
                "street": "Main St",
                "zip": "12345"
            }
        });

        let schema = infer_schema(&json, "test.v1").unwrap();
        assert_eq!(schema.fields["address"].field_type, FieldType::Table);
        let nested = schema.fields["address"].fields.as_ref().unwrap();
        assert_eq!(nested["street"].field_type, FieldType::String);
    }

    #[test]
    fn test_infer_all_optional() {
        let json: serde_json::Value = serde_json::json!({ "name": "X" });
        let schema = infer_schema(&json, "test.v1").unwrap();
        assert!(!schema.fields["name"].required);
    }

    #[test]
    fn test_infer_preserves_order() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
            "zebra": "z",
            "alpha": "a",
            "middle": "m"
        }"#,
        )
        .unwrap();

        let schema = infer_schema(&json, "test.v1").unwrap();
        let keys: Vec<&String> = schema.fields.keys().collect();
        assert_eq!(keys, &["zebra", "alpha", "middle"]);
    }
}
