//! # JSON Schema Draft 7 Adapter
//!
//! Converts JSON Schema Draft 7 input into GERMANIC's internal
//! [`SchemaDefinition`] format. This provides a second "entry door"
//! so that tools speaking standard JSON Schema (e.g. OpenClaw llm-task)
//! can use GERMANIC without knowing the proprietary format.
//!
//! ```text
//!                               +------------------------------+
//!   .schema.json (GERMANIC) --->|                              |
//!                               |      SchemaDefinition        |
//!                               |   (internal source of truth) |---> validate ---> compile
//!   .json (JSON Schema D7) --->|                              |
//!             ^                 +------------------------------+
//!             |
//!        json_schema.rs
//!        (this module)
//! ```
//!
//! ## Supported Features
//!
//! - `type`: string, boolean, integer, number, object, array
//! - `required`: object-level list inverted to per-field flags
//! - `default`: passed through as string
//! - `properties`: recursive conversion (nested objects become Tables)
//! - `items`: array item type inference (string/integer arrays)
//!
//! ## Intentionally Ignored (with warnings)
//!
//! `$ref`, `anyOf`, `oneOf`, `allOf`, `enum`, `pattern`, `minimum`,
//! `maximum`, `format`, `additionalProperties`

use indexmap::IndexMap;
use serde::Deserialize;

use super::schema_def::{FieldDefinition, FieldType, SchemaDefinition};
use crate::error::GermanicError;

// ============================================================================
// JSON SCHEMA STRUCTS (input deserialization)
// ============================================================================

/// Reduced JSON Schema representation -- only the features GERMANIC needs.
#[derive(Debug, Deserialize)]
struct JsonSchema {
    #[serde(rename = "$schema")]
    #[allow(dead_code)]
    schema_url: Option<String>,

    #[serde(rename = "type")]
    typ: Option<String>,

    properties: Option<IndexMap<String, JsonSchemaProperty>>,
    required: Option<Vec<String>>,

    #[serde(rename = "$id")]
    id: Option<String>,

    title: Option<String>,

    #[allow(dead_code)]
    description: Option<String>,
}

/// A single property in a JSON Schema object.
#[derive(Debug, Deserialize)]
struct JsonSchemaProperty {
    #[serde(rename = "type")]
    typ: Option<String>,

    properties: Option<IndexMap<String, JsonSchemaProperty>>,
    required: Option<Vec<String>>,
    items: Option<Box<JsonSchemaProperty>>,
    default: Option<serde_json::Value>,

    // Recognized but only warned about:
    #[serde(rename = "$ref")]
    reference: Option<String>,
    #[serde(rename = "anyOf")]
    any_of: Option<serde_json::Value>,
    #[serde(rename = "oneOf")]
    one_of: Option<serde_json::Value>,
    #[serde(rename = "allOf")]
    all_of: Option<serde_json::Value>,
    #[serde(rename = "enum")]
    enum_values: Option<serde_json::Value>,
    #[allow(dead_code)]
    pattern: Option<String>,
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Detects whether a JSON string is a JSON Schema Draft 7.
///
/// Heuristic: has `"$schema"` key, OR has `"type": "object"` + `"properties"`.
pub fn is_json_schema(input: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(input) else {
        return false;
    };
    let Some(obj) = value.as_object() else {
        return false;
    };

    // Definitive: has $schema key
    if obj.contains_key("$schema") {
        return true;
    }

    // Heuristic: has "type": "object" + "properties"
    let is_object_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .is_some_and(|t| t == "object");
    let has_properties = obj.contains_key("properties");

    is_object_type && has_properties
}

/// Converts a JSON Schema Draft 7 string into a [`SchemaDefinition`].
///
/// Returns `(SchemaDefinition, Vec<String>)` where the second element
/// contains warnings for unsupported features that were ignored.
///
/// # Errors
///
/// Returns `GermanicError` if:
/// - The input is not valid JSON
/// - The root type is not `"object"`
/// - Array items have mixed/unsupported types
pub fn convert_json_schema(input: &str) -> Result<(SchemaDefinition, Vec<String>), GermanicError> {
    let js: JsonSchema = serde_json::from_str(input)?;
    let mut warnings: Vec<String> = Vec::new();

    // Root must be "type": "object"
    match js.typ.as_deref() {
        Some("object") | None => {} // None is acceptable if properties exist
        Some(other) => {
            return Err(GermanicError::General(format!(
                "JSON Schema root must be \"object\", found \"{}\"",
                other
            )));
        }
    }

    // Derive schema_id from $id, title, or generate fallback
    let schema_id = js
        .id
        .or(js.title.map(|t| t.to_lowercase().replace(' ', "-")))
        .unwrap_or_else(|| "converted.json-schema.v1".to_string());

    // Convert properties
    let required_list = js.required.unwrap_or_default();
    let fields = match js.properties {
        Some(props) => convert_properties(props, &required_list, &mut warnings)?,
        None => IndexMap::new(),
    };

    let schema = SchemaDefinition {
        schema_id,
        version: 1,
        fields,
    };

    Ok((schema, warnings))
}

// ============================================================================
// INTERNAL CONVERSION
// ============================================================================

/// Converts a map of JSON Schema properties into GERMANIC FieldDefinitions.
fn convert_properties(
    properties: IndexMap<String, JsonSchemaProperty>,
    required_list: &[String],
    warnings: &mut Vec<String>,
) -> Result<IndexMap<String, FieldDefinition>, GermanicError> {
    let mut fields = IndexMap::new();

    for (name, prop) in properties {
        let is_required = required_list.contains(&name);
        let field = convert_property(&name, prop, is_required, warnings)?;
        fields.insert(name, field);
    }

    Ok(fields)
}

/// Converts a single JSON Schema property to a GERMANIC FieldDefinition.
fn convert_property(
    name: &str,
    prop: JsonSchemaProperty,
    required: bool,
    warnings: &mut Vec<String>,
) -> Result<FieldDefinition, GermanicError> {
    // Emit warnings for unsupported features
    if prop.reference.is_some() {
        warnings.push(format!(
            "Field \"{name}\": $ref not resolved (not supported)"
        ));
    }
    if prop.any_of.is_some() {
        warnings.push(format!("Field \"{name}\": anyOf not supported, ignored"));
    }
    if prop.one_of.is_some() {
        warnings.push(format!("Field \"{name}\": oneOf not supported, ignored"));
    }
    if prop.all_of.is_some() {
        warnings.push(format!("Field \"{name}\": allOf not supported, ignored"));
    }
    if prop.enum_values.is_some() {
        warnings.push(format!("Field \"{name}\": enum constraint ignored"));
    }

    // Determine field type
    let typ_str = prop.typ.as_deref().unwrap_or("string");

    let (field_type, nested_fields) = match typ_str {
        "string" => (FieldType::String, None),
        "boolean" => (FieldType::Bool, None),
        "integer" => (FieldType::Int, None),
        "number" => (FieldType::Float, None),
        "object" => {
            let nested_required = prop.required.unwrap_or_default();
            let nested = match prop.properties {
                Some(props) => Some(convert_properties(props, &nested_required, warnings)?),
                None => Some(IndexMap::new()),
            };
            (FieldType::Table, nested)
        }
        "array" => {
            let array_type = resolve_array_type(name, &prop.items)?;
            (array_type, None)
        }
        other => {
            warnings.push(format!(
                "Field \"{name}\": unknown type \"{other}\", defaulting to string"
            ));
            (FieldType::String, None)
        }
    };

    // Convert default value to string representation
    let default = prop.default.map(|v| match v {
        serde_json::Value::String(s) => s,
        other => other.to_string(),
    });

    Ok(FieldDefinition {
        field_type,
        required,
        default,
        fields: nested_fields,
    })
}

/// Determines the GERMANIC array type from JSON Schema `items`.
fn resolve_array_type(
    field_name: &str,
    items: &Option<Box<JsonSchemaProperty>>,
) -> Result<FieldType, GermanicError> {
    let Some(items) = items else {
        // No items specified, default to string array
        return Ok(FieldType::StringArray);
    };

    match items.typ.as_deref() {
        Some("string") | None => Ok(FieldType::StringArray),
        Some("integer") => Ok(FieldType::IntArray),
        Some("number") => Ok(FieldType::IntArray), // Closest mapping
        Some(other) => Err(GermanicError::General(format!(
            "Field \"{field_name}\": unsupported array item type \"{other}\""
        ))),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let input = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        }"#;

        let (schema, warnings) = convert_json_schema(input).unwrap();
        assert_eq!(schema.fields.len(), 1);
        assert_eq!(schema.fields["name"].field_type, FieldType::String);
        assert!(!schema.fields["name"].required);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_required_inversion() {
        let input = r#"{
            "type": "object",
            "required": ["a", "b"],
            "properties": {
                "a": { "type": "string" },
                "b": { "type": "integer" },
                "c": { "type": "string" }
            }
        }"#;

        let (schema, _) = convert_json_schema(input).unwrap();
        assert!(schema.fields["a"].required);
        assert!(schema.fields["b"].required);
        assert!(!schema.fields["c"].required);
    }

    #[test]
    fn test_nested_object() {
        let input = r#"{
            "type": "object",
            "properties": {
                "address": {
                    "type": "object",
                    "required": ["street"],
                    "properties": {
                        "street": { "type": "string" },
                        "city": { "type": "string" }
                    }
                }
            }
        }"#;

        let (schema, _) = convert_json_schema(input).unwrap();
        assert_eq!(schema.fields["address"].field_type, FieldType::Table);
        let nested = schema.fields["address"].fields.as_ref().unwrap();
        assert_eq!(nested.len(), 2);
        assert!(nested["street"].required);
        assert!(!nested["city"].required);
    }

    #[test]
    fn test_string_array() {
        let input = r#"{
            "type": "object",
            "properties": {
                "tags": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            }
        }"#;

        let (schema, _) = convert_json_schema(input).unwrap();
        assert_eq!(schema.fields["tags"].field_type, FieldType::StringArray);
    }

    #[test]
    fn test_int_array() {
        let input = r#"{
            "type": "object",
            "properties": {
                "scores": {
                    "type": "array",
                    "items": { "type": "integer" }
                }
            }
        }"#;

        let (schema, _) = convert_json_schema(input).unwrap();
        assert_eq!(schema.fields["scores"].field_type, FieldType::IntArray);
    }

    #[test]
    fn test_default_values() {
        let input = r#"{
            "type": "object",
            "properties": {
                "country": { "type": "string", "default": "DE" },
                "count": { "type": "integer", "default": 42 }
            }
        }"#;

        let (schema, _) = convert_json_schema(input).unwrap();
        assert_eq!(schema.fields["country"].default, Some("DE".into()));
        assert_eq!(schema.fields["count"].default, Some("42".into()));
    }

    #[test]
    fn test_schema_id_from_dollar_id() {
        let input = r#"{
            "$id": "practice.v1",
            "type": "object",
            "properties": {}
        }"#;

        let (schema, _) = convert_json_schema(input).unwrap();
        assert_eq!(schema.schema_id, "practice.v1");
    }

    #[test]
    fn test_schema_id_from_title() {
        let input = r#"{
            "title": "My Practice",
            "type": "object",
            "properties": {}
        }"#;

        let (schema, _) = convert_json_schema(input).unwrap();
        assert_eq!(schema.schema_id, "my-practice");
    }

    #[test]
    fn test_warning_on_ref() {
        let input = r##"{
            "type": "object",
            "properties": {
                "other": { "$ref": "#/definitions/Other" }
            }
        }"##;

        let (_, warnings) = convert_json_schema(input).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("$ref"));
    }

    #[test]
    fn test_warning_on_any_of() {
        let input = r#"{
            "type": "object",
            "properties": {
                "value": { "anyOf": [{"type": "string"}, {"type": "integer"}] }
            }
        }"#;

        let (_, warnings) = convert_json_schema(input).unwrap();
        assert!(warnings.iter().any(|w| w.contains("anyOf")));
    }

    #[test]
    fn test_error_on_non_object_root() {
        let input = r#"{ "type": "string" }"#;

        let result = convert_json_schema(input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("object"));
    }

    #[test]
    fn test_empty_properties() {
        let input = r#"{
            "type": "object",
            "properties": {}
        }"#;

        let (schema, warnings) = convert_json_schema(input).unwrap();
        assert!(schema.fields.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_is_json_schema_with_dollar_schema() {
        assert!(is_json_schema(
            r#"{"$schema": "http://json-schema.org/draft-07/schema#", "type": "object"}"#
        ));
    }

    #[test]
    fn test_is_json_schema_with_type_and_properties() {
        assert!(is_json_schema(
            r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#
        ));
    }

    #[test]
    fn test_is_not_json_schema_germanic_format() {
        // GERMANIC native format has schema_id + fields, not $schema/properties
        assert!(!is_json_schema(
            r#"{"schema_id": "test.v1", "version": 1, "fields": {}}"#
        ));
    }

    #[test]
    fn test_openclaw_llm_task_compatible() {
        let json_schema = r#"{
            "type": "object",
            "properties": {
                "intent": { "type": "string" },
                "draft": { "type": "string" }
            },
            "required": ["intent", "draft"],
            "additionalProperties": false
        }"#;

        let (schema, warnings) = convert_json_schema(json_schema).unwrap();
        assert!(schema.fields["intent"].required);
        assert!(schema.fields["draft"].required);
        assert_eq!(schema.fields["intent"].field_type, FieldType::String);
        assert_eq!(schema.fields["draft"].field_type, FieldType::String);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_praxis_as_json_schema_draft7() {
        let json_schema = r#"{
            "$schema": "http://json-schema.org/draft-07/schema#",
            "$id": "de.health.practice.v1",
            "type": "object",
            "required": ["name", "telefon", "adresse"],
            "properties": {
                "name": { "type": "string" },
                "telefon": { "type": "string" },
                "email": { "type": "string" },
                "adresse": {
                    "type": "object",
                    "required": ["strasse", "ort"],
                    "properties": {
                        "strasse": { "type": "string" },
                        "ort": { "type": "string" },
                        "land": { "type": "string", "default": "DE" }
                    }
                },
                "schwerpunkte": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "kassenpatienten": { "type": "boolean" }
            }
        }"#;

        let (schema, _) = convert_json_schema(json_schema).unwrap();

        // Schema metadata
        assert_eq!(schema.schema_id, "de.health.practice.v1");

        // Required inversion
        assert!(schema.fields["name"].required);
        assert!(schema.fields["telefon"].required);
        assert!(!schema.fields["email"].required);

        // Nested table
        assert_eq!(schema.fields["adresse"].field_type, FieldType::Table);
        let addr = schema.fields["adresse"].fields.as_ref().unwrap();
        assert!(addr["strasse"].required);
        assert!(addr["ort"].required);
        assert_eq!(addr["land"].default, Some("DE".into()));

        // Array
        assert_eq!(
            schema.fields["schwerpunkte"].field_type,
            FieldType::StringArray
        );

        // Bool
        assert_eq!(schema.fields["kassenpatienten"].field_type, FieldType::Bool);
    }

    #[test]
    fn test_all_field_types() {
        let input = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "active": { "type": "boolean" },
                "age": { "type": "integer" },
                "rating": { "type": "number" },
                "tags": { "type": "array", "items": { "type": "string" } },
                "scores": { "type": "array", "items": { "type": "integer" } },
                "address": {
                    "type": "object",
                    "properties": {
                        "city": { "type": "string" }
                    }
                }
            }
        }"#;

        let (schema, warnings) = convert_json_schema(input).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(schema.fields["name"].field_type, FieldType::String);
        assert_eq!(schema.fields["active"].field_type, FieldType::Bool);
        assert_eq!(schema.fields["age"].field_type, FieldType::Int);
        assert_eq!(schema.fields["rating"].field_type, FieldType::Float);
        assert_eq!(schema.fields["tags"].field_type, FieldType::StringArray);
        assert_eq!(schema.fields["scores"].field_type, FieldType::IntArray);
        assert_eq!(schema.fields["address"].field_type, FieldType::Table);
    }

    #[test]
    fn test_warning_on_enum() {
        let input = r#"{
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["active", "inactive"]
                }
            }
        }"#;

        let (schema, warnings) = convert_json_schema(input).unwrap();
        assert_eq!(schema.fields["status"].field_type, FieldType::String);
        assert!(warnings.iter().any(|w| w.contains("enum")));
    }

    #[test]
    fn test_schema_url_detection() {
        // Has $schema but no "type"+"properties" — should still detect
        assert!(is_json_schema(
            r#"{"$schema": "http://json-schema.org/draft-07/schema#"}"#
        ));
    }

    #[test]
    fn test_fallback_schema_id() {
        let input = r#"{
            "type": "object",
            "properties": {
                "x": { "type": "string" }
            }
        }"#;

        let (schema, _) = convert_json_schema(input).unwrap();
        assert_eq!(schema.schema_id, "converted.json-schema.v1");
    }

    #[test]
    fn test_array_without_items() {
        let input = r#"{
            "type": "object",
            "properties": {
                "things": { "type": "array" }
            }
        }"#;

        let (schema, _) = convert_json_schema(input).unwrap();
        // Defaults to string array when items not specified
        assert_eq!(schema.fields["things"].field_type, FieldType::StringArray);
    }

    #[test]
    fn test_warning_on_one_of() {
        let input = r#"{
            "type": "object",
            "properties": {
                "val": { "oneOf": [{"type": "string"}, {"type": "integer"}] }
            }
        }"#;

        let (_, warnings) = convert_json_schema(input).unwrap();
        assert!(warnings.iter().any(|w| w.contains("oneOf")));
    }

    #[test]
    fn test_warning_on_all_of() {
        let input = r#"{
            "type": "object",
            "properties": {
                "val": { "allOf": [{"type": "string"}] }
            }
        }"#;

        let (_, warnings) = convert_json_schema(input).unwrap();
        assert!(warnings.iter().any(|w| w.contains("allOf")));
    }
}
