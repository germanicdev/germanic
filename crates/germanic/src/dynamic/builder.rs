//! # Dynamic FlatBuffer Builder
//!
//! Builds FlatBuffer bytes at runtime from a SchemaDefinition + JSON data.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                  DYNAMIC FLATBUFFER BUILDING                    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  SchemaDefinition        JSON data         FlatBufferBuilder    │
//! │  ┌──────────────┐       ┌──────────┐      ┌──────────────┐    │
//! │  │ fields[0..n] │       │ values   │      │              │    │
//! │  │ with types   │ ──┐   │          │ ──┐   │ push_slot()  │    │
//! │  │ and order    │   │   │          │   │   │ per field    │    │
//! │  └──────────────┘   │   └──────────┘   │   └──────────────┘    │
//! │                     │                  │          │             │
//! │                     └──────┬───────────┘          │             │
//! │                            │                      ▼             │
//! │                    ┌───────▼────────┐     ┌──────────────┐     │
//! │                    │  build_table() │     │  .grm bytes  │     │
//! │                    │  (recursive)   │────►│              │     │
//! │                    └────────────────┘     └──────────────┘     │
//! │                                                                 │
//! │  vtable slot = 4 + (2 × field_index)                           │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use crate::dynamic::schema_def::{FieldDefinition, FieldType, SchemaDefinition};
use crate::error::GermanicError;
use flatbuffers::FlatBufferBuilder;
use indexmap::IndexMap;

/// Builds FlatBuffer bytes from a schema definition and JSON data.
///
/// Returns the raw FlatBuffer payload (WITHOUT .grm header).
/// The caller wraps it with GrmHeader to produce the final .grm file.
pub fn build_flatbuffer(
    schema: &SchemaDefinition,
    data: &serde_json::Value,
) -> Result<Vec<u8>, GermanicError> {
    let obj = data.as_object().ok_or_else(|| {
        GermanicError::General("Root data must be a JSON object".into())
    })?;

    let mut builder = FlatBufferBuilder::with_capacity(1024);

    let root = build_table(&mut builder, &schema.fields, obj)?;

    builder.finish_minimal(root);
    Ok(builder.finished_data().to_vec())
}

/// A field value prepared for insertion into the FlatBuffer.
///
/// Offset types are stored as raw u32 values to avoid lifetime issues
/// with FlatBufferBuilder's generic WIPOffset types. All WIPOffset<T>
/// variants are simple u32 wrappers, so this is safe.
enum PreparedField {
    /// Field not in data and no default — skip (not in vtable).
    Absent,
    /// String offset (raw u32 from WIPOffset).
    Offset(u32),
    /// Boolean value + default.
    Bool(bool, bool),
    /// 32-bit integer value + default.
    Int(i32, i32),
    /// 32-bit float value + default.
    Float(f32, f32),
}

/// Recursively builds a FlatBuffer table from field definitions and JSON data.
///
/// CRITICAL: Must follow inside-out order:
/// 1. Strings and vectors (offsets to buffer end)
/// 2. Nested tables (which themselves follow the same pattern)
/// 3. Then the current table's vtable slots
fn build_table(
    builder: &mut FlatBufferBuilder<'_>,
    fields: &IndexMap<String, FieldDefinition>,
    data: &serde_json::Map<String, serde_json::Value>,
) -> Result<flatbuffers::WIPOffset<flatbuffers::TableFinishedWIPOffset>, GermanicError> {
    // Phase 1: Pre-create all offset values (strings, vectors, nested tables)
    // We must create these BEFORE starting the table.
    let mut prepared: IndexMap<String, PreparedField> = IndexMap::new();

    for (name, def) in fields {
        let value = data.get(name);
        let prep = prepare_field(builder, def, value)?;
        prepared.insert(name.clone(), prep);
    }

    // Phase 2: Start table and push slots
    let table_start = builder.start_table();

    for (index, (name, _def)) in fields.iter().enumerate() {
        let voffset = 4 + (2 * index) as u16;
        let prep = &prepared[name];

        match prep {
            PreparedField::Absent => {
                // Field not in data and no default — skip (not in vtable)
            }
            PreparedField::Offset(raw) => {
                builder.push_slot_always::<flatbuffers::WIPOffset<&str>>(
                    voffset,
                    flatbuffers::WIPOffset::new(*raw),
                );
            }
            PreparedField::Bool(val, default) => {
                builder.push_slot::<bool>(voffset, *val, *default);
            }
            PreparedField::Int(val, default) => {
                builder.push_slot::<i32>(voffset, *val, *default);
            }
            PreparedField::Float(val, default) => {
                builder.push_slot::<f32>(voffset, *val, *default);
            }
        }
    }

    Ok(builder.end_table(table_start))
}

/// Prepares a single field value for FlatBuffer insertion.
fn prepare_field(
    builder: &mut FlatBufferBuilder<'_>,
    def: &FieldDefinition,
    value: Option<&serde_json::Value>,
) -> Result<PreparedField, GermanicError> {
    let Some(value) = value else {
        // Field not present — check for default
        return Ok(match &def.default {
            Some(d) => match def.field_type {
                FieldType::String => {
                    PreparedField::Offset(builder.create_string(d).value())
                }
                FieldType::Bool => PreparedField::Bool(d.parse().unwrap_or(false), false),
                FieldType::Int => PreparedField::Int(d.parse().unwrap_or(0), 0),
                FieldType::Float => PreparedField::Float(d.parse().unwrap_or(0.0), 0.0),
                _ => PreparedField::Absent,
            },
            None => PreparedField::Absent,
        });
    };

    match def.field_type {
        FieldType::String => {
            let s = value.as_str().unwrap_or("");
            Ok(PreparedField::Offset(builder.create_string(s).value()))
        }

        FieldType::Bool => {
            let v = value.as_bool().unwrap_or(false);
            let default: bool = def
                .default
                .as_ref()
                .and_then(|d| d.parse().ok())
                .unwrap_or(false);
            Ok(PreparedField::Bool(v, default))
        }

        FieldType::Int => {
            let v = value.as_i64().unwrap_or(0) as i32;
            let default: i32 = def
                .default
                .as_ref()
                .and_then(|d| d.parse().ok())
                .unwrap_or(0);
            Ok(PreparedField::Int(v, default))
        }

        FieldType::Float => {
            let v = value.as_f64().unwrap_or(0.0) as f32;
            let default: f32 = def
                .default
                .as_ref()
                .and_then(|d| d.parse().ok())
                .unwrap_or(0.0);
            Ok(PreparedField::Float(v, default))
        }

        FieldType::StringArray => match value.as_array() {
            Some(arr) if !arr.is_empty() => {
                let offsets: Vec<_> = arr
                    .iter()
                    .map(|v| builder.create_string(v.as_str().unwrap_or("")))
                    .collect();
                let vec_offset = builder.create_vector(&offsets);
                Ok(PreparedField::Offset(vec_offset.value()))
            }
            _ => Ok(PreparedField::Absent),
        },

        FieldType::IntArray => match value.as_array() {
            Some(arr) if !arr.is_empty() => {
                let values: Vec<i32> = arr
                    .iter()
                    .map(|v| v.as_i64().unwrap_or(0) as i32)
                    .collect();
                let vec_offset = builder.create_vector(&values);
                Ok(PreparedField::Offset(vec_offset.value()))
            }
            _ => Ok(PreparedField::Absent),
        },

        FieldType::Table => {
            let nested_fields = def.fields.as_ref().ok_or_else(|| {
                GermanicError::General("Table field has no nested field definitions".into())
            })?;

            match value.as_object() {
                Some(obj) => {
                    let table_offset = build_table(builder, nested_fields, obj)?;
                    Ok(PreparedField::Offset(table_offset.value()))
                }
                None => Ok(PreparedField::Absent),
            }
        }
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

    fn minimal_schema() -> SchemaDefinition {
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
        SchemaDefinition {
            schema_id: "test.v1".into(),
            version: 1,
            fields,
        }
    }

    #[test]
    fn test_build_minimal() {
        let schema = minimal_schema();
        let data = serde_json::json!({ "name": "Hello" });
        let bytes = build_flatbuffer(&schema, &data).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_build_with_bool() {
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
            "active".into(),
            FieldDefinition {
                field_type: FieldType::Bool,
                required: false,
                default: Some("false".into()),
                fields: None,
            },
        );

        let schema = SchemaDefinition {
            schema_id: "test.v1".into(),
            version: 1,
            fields,
        };

        let data = serde_json::json!({ "name": "Test", "active": true });
        let bytes = build_flatbuffer(&schema, &data).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_build_with_nested() {
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
            "address".into(),
            FieldDefinition {
                field_type: FieldType::Table,
                required: true,
                default: None,
                fields: Some(addr_fields),
            },
        );

        let schema = SchemaDefinition {
            schema_id: "test.v1".into(),
            version: 1,
            fields,
        };

        let data = serde_json::json!({
            "name": "Test",
            "address": {
                "street": "Main St",
                "city": "Berlin"
            }
        });

        let bytes = build_flatbuffer(&schema, &data).unwrap();
        assert!(!bytes.is_empty());
        assert!(bytes.len() > 20);
    }

    #[test]
    fn test_build_with_string_array() {
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
            "tags".into(),
            FieldDefinition {
                field_type: FieldType::StringArray,
                required: false,
                default: None,
                fields: None,
            },
        );

        let schema = SchemaDefinition {
            schema_id: "test.v1".into(),
            version: 1,
            fields,
        };

        let data = serde_json::json!({ "name": "Test", "tags": ["a", "b", "c"] });
        let bytes = build_flatbuffer(&schema, &data).unwrap();
        assert!(!bytes.is_empty());
    }
}
