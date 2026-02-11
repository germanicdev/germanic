//! # Dynamic Compilation Mode (Weg 3)
//!
//! Compiles JSON to .grm without Rust code or FlatBuffer knowledge.
//!
//! ## Workflow
//!
//! ```text
//! ┌────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────┐
//! │ example    │     │ .schema.json │     │  data.json   │     │ .grm     │
//! │ .json      │────►│ (inferred)   │────►│ + schema     │────►│ (binary) │
//! └────────────┘     └──────────────┘     └──────────────┘     └──────────┘
//!   germanic init      user edits         germanic compile
//! ```

pub mod builder;
pub mod infer;
pub mod json_schema;
pub mod schema_def;
pub mod validate;

use crate::error::{GermanicError, GermanicResult};
use crate::types::GrmHeader;
use std::path::Path;

/// Compiles JSON data to .grm using a schema definition file.
///
/// This is the main entry point for dynamic compilation (Weg 3).
/// Accepts both GERMANIC `.schema.json` and JSON Schema Draft 7 files.
/// Auto-detection chooses the right parser transparently.
///
/// ## Steps
/// 1. Load schema definition (auto-detect format)
/// 2. Load and parse input JSON
/// 3. Validate data against schema
/// 4. Build FlatBuffer payload dynamically
/// 5. Prepend .grm header
///
/// ## Returns
///
/// `(grm_bytes, warnings)` — warnings list unsupported JSON Schema features.
pub fn compile_dynamic(schema_path: &Path, data_path: &Path) -> GermanicResult<Vec<u8>> {
    // 1. Load schema (auto-detect JSON Schema Draft 7 vs GERMANIC native)
    let (schema, _warnings) = load_schema_auto(schema_path)?;

    // 2. Load data
    let json_str = std::fs::read_to_string(data_path)?;
    let data: serde_json::Value = serde_json::from_str(&json_str)?;

    // 3. Validate
    validate::validate_against_schema(&schema, &data).map_err(GermanicError::Validation)?;

    // 4. Build FlatBuffer
    let payload = builder::build_flatbuffer(&schema, &data)?;

    // 5. Prepend header
    let header = GrmHeader::new(&schema.schema_id);
    let header_bytes = header.to_bytes();

    let mut output = Vec::with_capacity(header_bytes.len() + payload.len());
    output.extend_from_slice(&header_bytes);
    output.extend_from_slice(&payload);

    Ok(output)
}

/// Compiles JSON data to .grm using a schema definition (in-memory).
///
/// Same as compile_dynamic but takes pre-loaded schema and data.
pub fn compile_dynamic_from_values(
    schema: &schema_def::SchemaDefinition,
    data: &serde_json::Value,
) -> GermanicResult<Vec<u8>> {
    // 1. Validate
    validate::validate_against_schema(schema, data).map_err(GermanicError::Validation)?;

    // 2. Build FlatBuffer
    let payload = builder::build_flatbuffer(schema, data)?;

    // 3. Prepend header
    let header = GrmHeader::new(&schema.schema_id);
    let header_bytes = header.to_bytes();

    let mut output = Vec::with_capacity(header_bytes.len() + payload.len());
    output.extend_from_slice(&header_bytes);
    output.extend_from_slice(&payload);

    Ok(output)
}

/// Loads a schema from file with auto-detection of format.
///
/// Detects whether the file is JSON Schema Draft 7 or GERMANIC native
/// format and parses accordingly. Returns the schema and any warnings
/// (only relevant for JSON Schema conversion).
pub fn load_schema_auto(
    schema_path: &Path,
) -> GermanicResult<(schema_def::SchemaDefinition, Vec<String>)> {
    let content = std::fs::read_to_string(schema_path)?;

    if json_schema::is_json_schema(&content) {
        json_schema::convert_json_schema(&content)
    } else {
        let schema: schema_def::SchemaDefinition = serde_json::from_str(&content)?;
        Ok((schema, Vec::new()))
    }
}
