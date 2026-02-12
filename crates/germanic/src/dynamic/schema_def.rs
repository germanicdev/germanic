//! # Schema Definition
//!
//! Runtime schema definitions for dynamic compilation.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    SCHEMA DEFINITION                            │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │   .schema.json                    In Memory                     │
//! │   ┌──────────────────┐           ┌──────────────────┐          │
//! │   │ schema_id: "..." │  deser.   │ SchemaDefinition │          │
//! │   │ version: 1       │ ──────►   │   .schema_id     │          │
//! │   │ fields: {        │           │   .version       │          │
//! │   │   "name": {...}  │           │   .fields: IndexMap<        │
//! │   │   "addr": {...}  │           │     String,                 │
//! │   │ }                │           │     FieldDefinition>        │
//! │   └──────────────────┘           └──────────────────┘          │
//! │                                                                 │
//! │   Field order in IndexMap = vtable slot order                   │
//! │   Slot formula: voffset = 4 + (2 × field_index)                │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Complete schema definition loaded from a .schema.json file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    /// Unique schema identifier.
    /// Format: "namespace.domain.name.vN"
    /// Example: "de.dining.restaurant.v1"
    pub schema_id: String,

    /// Schema version (1-255).
    pub version: u8,

    /// Ordered map of field name → field definition.
    /// ORDER MATTERS: field position determines FlatBuffer vtable slot.
    pub fields: IndexMap<String, FieldDefinition>,
}

/// Definition of a single field within a schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// The field type.
    #[serde(rename = "type")]
    pub field_type: FieldType,

    /// Whether this field is required (must be non-empty).
    #[serde(default)]
    pub required: bool,

    /// Default value as JSON string (e.g. "DE", "true", "42").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Nested fields (only for FieldType::Table).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<IndexMap<String, FieldDefinition>>,
}

/// Supported field types for dynamic schemas.
///
/// Maps directly to FlatBuffer scalar/offset types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    /// UTF-8 string → FlatBuffer string offset
    #[serde(rename = "string")]
    String,

    /// Boolean → FlatBuffer bool (1 byte)
    #[serde(rename = "bool")]
    Bool,

    /// 32-bit signed integer → FlatBuffer int32
    #[serde(rename = "int")]
    Int,

    /// 32-bit float → FlatBuffer float32
    #[serde(rename = "float")]
    Float,

    /// Vector of strings → FlatBuffer vector of string offsets
    #[serde(rename = "[string]")]
    StringArray,

    /// Vector of integers → FlatBuffer vector of int32
    #[serde(rename = "[int]")]
    IntArray,

    /// Nested table → FlatBuffer table offset
    #[serde(rename = "table")]
    Table,
}

impl SchemaDefinition {
    /// Loads a schema definition from a .schema.json file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, crate::error::GermanicError> {
        let content = std::fs::read_to_string(path)?;
        let schema: Self = serde_json::from_str(&content)?;
        Ok(schema)
    }

    /// Saves the schema definition to a .schema.json file.
    pub fn to_file(&self, path: &std::path::Path) -> Result<(), crate::error::GermanicError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Counts total fields (including nested).
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_restaurant_schema() -> SchemaDefinition {
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
            "cuisine".into(),
            FieldDefinition {
                field_type: FieldType::String,
                required: false,
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
        fields.insert(
            "tags".into(),
            FieldDefinition {
                field_type: FieldType::StringArray,
                required: false,
                default: None,
                fields: None,
            },
        );

        let mut addr_fields = IndexMap::new();
        addr_fields.insert(
            "street".into(),
            FieldDefinition {
                field_type: FieldType::String,
                required: true,
                default: None,
                fields: None,
            },
        );
        addr_fields.insert(
            "city".into(),
            FieldDefinition {
                field_type: FieldType::String,
                required: true,
                default: None,
                fields: None,
            },
        );
        addr_fields.insert(
            "country".into(),
            FieldDefinition {
                field_type: FieldType::String,
                required: false,
                default: Some("DE".into()),
                fields: None,
            },
        );

        fields.insert(
            "address".into(),
            FieldDefinition {
                field_type: FieldType::Table,
                required: true,
                default: None,
                fields: Some(addr_fields),
            },
        );

        SchemaDefinition {
            schema_id: "de.dining.restaurant.v1".into(),
            version: 1,
            fields,
        }
    }

    #[test]
    fn test_schema_serialize_roundtrip() {
        let schema = sample_restaurant_schema();
        let json = serde_json::to_string_pretty(&schema).unwrap();
        let parsed: SchemaDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.schema_id, "de.dining.restaurant.v1");
        assert_eq!(parsed.fields.len(), 5);
        // Verify order preserved
        let keys: Vec<&String> = parsed.fields.keys().collect();
        assert_eq!(keys, &["name", "cuisine", "rating", "tags", "address"]);
    }

    #[test]
    fn test_field_type_serde() {
        let json = r#"{"type": "string", "required": true}"#;
        let field: FieldDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(field.field_type, FieldType::String);
        assert!(field.required);

        let json = r#"{"type": "[string]"}"#;
        let field: FieldDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(field.field_type, FieldType::StringArray);
    }

    #[test]
    fn test_nested_table_fields() {
        let schema = sample_restaurant_schema();
        let addr = &schema.fields["address"];
        assert_eq!(addr.field_type, FieldType::Table);
        let nested = addr.fields.as_ref().unwrap();
        assert_eq!(nested.len(), 3);
        assert!(nested["street"].required);
    }
}
