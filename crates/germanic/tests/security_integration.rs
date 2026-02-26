//! # Security Integration Tests — Regression Guards
//!
//! These tests prove that security fixes (pre_validate, exit codes, etc.)
//! are actually wired into the compilation and CLI pipelines.
//!
//! If someone removes a `pre_validate()` call or changes an exit code,
//! these tests will fail immediately.
//!
//! ```text
//! GROUP 1: compile_dynamic() + pre_validate pipeline
//! GROUP 2: compile_dynamic_from_values() + pre_validate_value pipeline
//! GROUP 3: CLI exit codes (validate, inspect, compile)
//! GROUP 4: GrmHeader::to_bytes() returns Result (compile-time guard)
//! ```

// ============================================================================
// GROUP 1: compile_dynamic() rejects oversized input (pipeline integration)
// ============================================================================

/// Proves that `pre_validate()` is wired into `compile_dynamic()`.
///
/// If someone removes the size check from `compile_dynamic()`,
/// this test will fail.
#[test]
fn compile_dynamic_rejects_oversized_input() {
    use germanic::dynamic::compile_dynamic;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Minimal schema — accepts arbitrary string fields
    let schema_json = r#"{
        "schema_id": "test.oversized.v1",
        "version": 1,
        "fields": {
            "name": { "type": "string", "required": false }
        }
    }"#;
    let mut schema_file = NamedTempFile::with_suffix(".schema.json").unwrap();
    schema_file.write_all(schema_json.as_bytes()).unwrap();

    // Data > 5 MB: many fields with 1000-char values
    let mut data = String::from("{");
    for i in 0..6000 {
        if i > 0 {
            data.push(',');
        }
        data.push_str(&format!(r#""f{}":"{}""#, i, "x".repeat(1000)));
    }
    data.push('}');
    assert!(data.len() > 5_242_880, "Test data must be > 5 MB");

    let mut data_file = NamedTempFile::with_suffix(".json").unwrap();
    data_file.write_all(data.as_bytes()).unwrap();

    let result = compile_dynamic(schema_file.path(), data_file.path());
    assert!(result.is_err(), "Oversized input must be rejected");

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("input size") || err_msg.contains("exceeds maximum"),
        "Error must mention size limit, was: {}",
        err_msg
    );
}

/// Boundary test: input at exactly MAX_INPUT_SIZE must NOT trigger size rejection.
///
/// Uses many small fields (each well under MAX_STRING_LENGTH) to reach
/// the boundary without triggering per-string limits.
#[test]
fn compile_dynamic_boundary_at_limit() {
    use germanic::dynamic::compile_dynamic;
    use germanic::pre_validate::MAX_INPUT_SIZE;
    use std::io::Write;
    use tempfile::NamedTempFile;

    let schema_json = r#"{
        "schema_id": "test.boundary.v1",
        "version": 1,
        "fields": {
            "data": { "type": "string", "required": false }
        }
    }"#;
    let mut schema_file = NamedTempFile::with_suffix(".schema.json").unwrap();
    schema_file.write_all(schema_json.as_bytes()).unwrap();

    // Build JSON with many small fields, staying just under MAX_INPUT_SIZE.
    // Each value is 500 bytes — well under MAX_STRING_LENGTH (1 MB).
    let value = "a".repeat(500);
    let mut data = String::from("{");
    let mut i = 0;
    loop {
        let field = format!(r#""f{}":"{}""#, i, value);
        // +2 for potential comma and closing brace
        if data.len() + field.len() + 2 > MAX_INPUT_SIZE {
            break;
        }
        if i > 0 {
            data.push(',');
        }
        data.push_str(&field);
        i += 1;
    }
    data.push('}');
    assert!(
        data.len() <= MAX_INPUT_SIZE,
        "Test data must be <= {} bytes, was {}",
        MAX_INPUT_SIZE,
        data.len()
    );
    // Sanity: we should be reasonably close to the limit
    assert!(
        data.len() > MAX_INPUT_SIZE - 1000,
        "Test data should be close to the limit, was {} (limit {})",
        data.len(),
        MAX_INPUT_SIZE
    );

    let mut data_file = NamedTempFile::with_suffix(".json").unwrap();
    data_file.write_all(data.as_bytes()).unwrap();

    let result = compile_dynamic(schema_file.path(), data_file.path());

    // The result may fail due to schema validation (extra fields) — that's fine.
    // We only assert it does NOT fail due to input size.
    if let Err(ref e) = result {
        let msg = format!("{}", e);
        assert!(
            !msg.contains("input size") && !msg.contains("exceeds maximum"),
            "Input <= {} bytes must not be rejected for size, error was: {}",
            MAX_INPUT_SIZE,
            msg
        );
    }
}

// ============================================================================
// GROUP 2: compile_dynamic_from_values() rejects oversized values
// ============================================================================

/// Proves that `pre_validate_value()` is wired into `compile_dynamic_from_values()`.
///
/// If someone removes the pre_validate_value() call, this test will fail.
#[test]
fn compile_from_values_rejects_oversized_string() {
    use germanic::dynamic::compile_dynamic_from_values;
    use germanic::dynamic::schema_def::SchemaDefinition;
    use germanic::pre_validate::MAX_STRING_LENGTH;

    let schema_json = r#"{
        "schema_id": "test.string_limit.v1",
        "version": 1,
        "fields": {
            "name": { "type": "string", "required": false }
        }
    }"#;
    let schema: SchemaDefinition = serde_json::from_str(schema_json).unwrap();

    let big_string = "x".repeat(MAX_STRING_LENGTH + 1);
    let data = serde_json::json!({ "name": big_string });

    let result = compile_dynamic_from_values(&schema, &data);
    assert!(result.is_err(), "String > 1 MB must be rejected");

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("string length"),
        "Error must mention string length, was: {}",
        err_msg
    );
}

/// Proves that array size limits are enforced in the from_values pipeline.
#[test]
fn compile_from_values_rejects_oversized_array() {
    use germanic::dynamic::compile_dynamic_from_values;
    use germanic::dynamic::schema_def::SchemaDefinition;
    use germanic::pre_validate::MAX_ARRAY_ELEMENTS;

    let schema_json = r#"{
        "schema_id": "test.array_limit.v1",
        "version": 1,
        "fields": {
            "items": { "type": "[string]", "required": false }
        }
    }"#;
    let schema: SchemaDefinition = serde_json::from_str(schema_json).unwrap();

    let items: Vec<serde_json::Value> = (0..MAX_ARRAY_ELEMENTS + 1)
        .map(|i| serde_json::Value::String(format!("x{}", i)))
        .collect();
    let data = serde_json::json!({ "items": items });

    let result = compile_dynamic_from_values(&schema, &data);
    assert!(
        result.is_err(),
        "Array > {} elements must be rejected",
        MAX_ARRAY_ELEMENTS
    );

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("array has") || err_msg.contains("elements"),
        "Error must mention array size, was: {}",
        err_msg
    );
}

// ============================================================================
// GROUP 3: CLI exit codes
// ============================================================================

/// `germanic validate` must exit 1 on corrupt .grm file.
#[test]
fn cli_validate_exit_1_on_invalid_grm() {
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    let mut corrupt = NamedTempFile::with_suffix(".grm").unwrap();
    corrupt.write_all(b"this is not a grm file").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_germanic"))
        .args(["validate", corrupt.path().to_str().unwrap()])
        .output()
        .expect("Binary must be callable");

    assert!(
        !output.status.success(),
        "Exit code must be != 0 for invalid .grm, was: {}",
        output.status
    );
}

/// `germanic validate` must exit 0 on a valid .grm file.
#[test]
fn cli_validate_exit_0_on_valid_grm() {
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    // Step 1: Create valid practice JSON
    let valid_json = r#"{
        "name": "Dr. Test",
        "bezeichnung": "Allgemeinmedizin",
        "adresse": {
            "strasse": "Teststrasse",
            "hausnummer": "1",
            "plz": "12345",
            "ort": "Teststadt",
            "land": "DE"
        }
    }"#;
    let mut input = NamedTempFile::with_suffix(".json").unwrap();
    input.write_all(valid_json.as_bytes()).unwrap();

    let output_grm = NamedTempFile::with_suffix(".grm").unwrap();

    // Step 2: Compile to .grm
    let compile = Command::new(env!("CARGO_BIN_EXE_germanic"))
        .args([
            "compile",
            "--schema",
            "practice",
            "--input",
            input.path().to_str().unwrap(),
            "--output",
            output_grm.path().to_str().unwrap(),
        ])
        .output()
        .expect("Compile must work");
    assert!(
        compile.status.success(),
        "Compile must succeed, stderr: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Step 3: Validate the .grm
    let validate = Command::new(env!("CARGO_BIN_EXE_germanic"))
        .args(["validate", output_grm.path().to_str().unwrap()])
        .output()
        .expect("Validate must be callable");

    assert!(
        validate.status.success(),
        "Exit code must be 0 for valid .grm, was: {}.\nStderr: {}",
        validate.status,
        String::from_utf8_lossy(&validate.stderr)
    );
}

/// `germanic inspect` must exit 1 on corrupt .grm file.
#[test]
fn cli_inspect_exit_1_on_invalid_grm() {
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    let mut corrupt = NamedTempFile::with_suffix(".grm").unwrap();
    corrupt.write_all(b"corrupt").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_germanic"))
        .args(["inspect", corrupt.path().to_str().unwrap()])
        .output()
        .expect("Binary must be callable");

    assert!(
        !output.status.success(),
        "Exit code must be != 0 for corrupt .grm, was: {}",
        output.status
    );
}

/// `germanic compile` must reject oversized input with exit 1.
#[test]
fn cli_compile_rejects_oversized_input() {
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    // Create JSON > 5 MB
    let mut data = String::from("{");
    for i in 0..6000 {
        if i > 0 {
            data.push(',');
        }
        data.push_str(&format!(r#""f{}":"{}""#, i, "x".repeat(1000)));
    }
    data.push('}');
    assert!(data.len() > 5_242_880);

    let mut input = NamedTempFile::with_suffix(".json").unwrap();
    input.write_all(data.as_bytes()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_germanic"))
        .args([
            "compile",
            "--schema",
            "practice",
            "--input",
            input.path().to_str().unwrap(),
        ])
        .output()
        .expect("Binary must be callable");

    assert!(
        !output.status.success(),
        "Oversized input must produce exit != 0"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let all_output = format!("{}{}", stdout, stderr);
    assert!(
        all_output.contains("input size") || all_output.contains("exceeds maximum"),
        "Output must mention size limit, was:\nstdout: {}\nstderr: {}",
        stdout,
        stderr
    );
}

// ============================================================================
// GROUP 4: GrmHeader::to_bytes() returns Result (compile-time guard)
// ============================================================================

/// Compile-time regression guard: if someone changes `to_bytes()` back to
/// returning `Vec<u8>` instead of `Result<Vec<u8>, _>`, this test won't compile.
#[test]
fn header_to_bytes_returns_result() {
    use germanic::types::GrmHeader;

    let header = GrmHeader::new("test.v1");
    // This only compiles if to_bytes() returns Result<Vec<u8>, _>.
    let bytes: Result<Vec<u8>, _> = header.to_bytes();
    assert!(bytes.is_ok());
}
