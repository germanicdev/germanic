//! # Pre-Validation
//!
//! Schema-agnostic structural checks that run before any compilation.
//!
//! ```text
//! JSON string ──► pre_validate() ──► Schema-specific validation
//!                     │
//!                     ├── Input size limit
//!                     ├── Must be JSON object
//!                     ├── String length limits
//!                     ├── Array element limits
//!                     └── Nesting depth limit
//! ```
//!
//! Defense-in-depth: protects both the Library API (Static Mode)
//! and the CLI (Dynamic Mode) from oversized or deeply nested input.

/// Maximum total input size in bytes (5 MB).
pub const MAX_INPUT_SIZE: usize = 5_242_880;

/// Maximum allowed length for a single string value in bytes (1 MB).
pub const MAX_STRING_LENGTH: usize = 1_048_576;

/// Maximum allowed number of elements in an array.
pub const MAX_ARRAY_ELEMENTS: usize = 10_000;

/// Maximum nesting depth for objects/arrays.
pub const MAX_NESTING_DEPTH: usize = 32;

/// Schema-agnostic structural validation.
///
/// Checks the raw JSON input and parsed Value for size/depth violations.
/// Collects ALL errors (not fail-fast).
///
/// ## Example
///
/// ```rust,ignore
/// let value: serde_json::Value = serde_json::from_str(&json)?;
/// pre_validate(&json, &value)?;
/// ```
pub fn pre_validate(raw_json: &str, value: &serde_json::Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Check 1: Total input size
    if raw_json.len() > MAX_INPUT_SIZE {
        errors.push(format!(
            "input size {} bytes exceeds maximum of {} bytes",
            raw_json.len(),
            MAX_INPUT_SIZE
        ));
    }

    // Check 2: Must be a JSON object at root
    if !value.is_object() {
        errors.push(format!(
            "expected JSON object at root, found {}",
            value_type_name(value)
        ));
    }

    // Check 3: Recurse into the value tree
    check_value(value, "", &mut errors, 0);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Value-only structural validation (no raw-string size check).
///
/// Use when the raw JSON string is not available (e.g. pre-parsed `Value`).
/// Checks string lengths, array sizes, and nesting depth.
pub fn pre_validate_value(value: &serde_json::Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if !value.is_object() {
        errors.push(format!(
            "expected JSON object at root, found {}",
            value_type_name(value)
        ));
    }

    check_value(value, "", &mut errors, 0);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Recursively checks a JSON value for size/depth violations.
fn check_value(value: &serde_json::Value, path: &str, errors: &mut Vec<String>, depth: usize) {
    if depth > MAX_NESTING_DEPTH {
        errors.push(format!(
            "{}: nesting depth exceeds maximum of {}",
            if path.is_empty() { "(root)" } else { path },
            MAX_NESTING_DEPTH
        ));
        return;
    }

    match value {
        serde_json::Value::String(s) if s.len() > MAX_STRING_LENGTH => {
            errors.push(format!(
                "{}: string length {} exceeds maximum of {} bytes",
                if path.is_empty() { "(root)" } else { path },
                s.len(),
                MAX_STRING_LENGTH
            ));
        }
        serde_json::Value::Array(arr) => {
            if arr.len() > MAX_ARRAY_ELEMENTS {
                errors.push(format!(
                    "{}: array has {} elements, maximum is {}",
                    if path.is_empty() { "(root)" } else { path },
                    arr.len(),
                    MAX_ARRAY_ELEMENTS
                ));
            }
            for (i, item) in arr.iter().enumerate() {
                let item_path = format!("{}[{}]", if path.is_empty() { "(root)" } else { path }, i);
                check_value(item, &item_path, errors, depth + 1);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let field_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                check_value(val, &field_path, errors, depth + 1);
            }
        }
        _ => {}
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_validate_valid() {
        let json = r#"{"name": "Test", "value": 42}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(pre_validate(json, &value).is_ok());
    }

    #[test]
    fn test_pre_validate_not_object() {
        let json = "[1, 2, 3]";
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let err = pre_validate(json, &value).unwrap_err();
        assert!(err.iter().any(|e| e.contains("expected JSON object")));
    }

    #[test]
    fn test_pre_validate_string_too_long() {
        let long_string = "x".repeat(MAX_STRING_LENGTH + 1);
        let json = format!(r#"{{"name": "{}"}}"#, long_string);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let err = pre_validate(&json, &value).unwrap_err();
        assert!(err.iter().any(|e| e.contains("string length")));
    }

    #[test]
    fn test_pre_validate_array_too_large() {
        let elements: Vec<String> = (0..MAX_ARRAY_ELEMENTS + 1)
            .map(|i| format!("\"x{}\"", i))
            .collect();
        let json = format!(r#"{{"items": [{}]}}"#, elements.join(","));
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let err = pre_validate(&json, &value).unwrap_err();
        assert!(err.iter().any(|e| e.contains("array has")));
    }

    #[test]
    fn test_pre_validate_nesting_too_deep() {
        // Build 33 levels of nesting
        let mut json = String::from(r#"{"a":"ok"}"#);
        for _ in 0..MAX_NESTING_DEPTH + 1 {
            json = format!(r#"{{"nested": {}}}"#, json);
        }
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let err = pre_validate(&json, &value).unwrap_err();
        assert!(err.iter().any(|e| e.contains("nesting depth")));
    }

    #[test]
    fn test_pre_validate_collects_all_errors() {
        let long_string = "x".repeat(MAX_STRING_LENGTH + 1);
        let elements: Vec<String> = (0..MAX_ARRAY_ELEMENTS + 1)
            .map(|i| format!("\"x{}\"", i))
            .collect();
        let json = format!(
            r#"{{"big": "{}", "many": [{}]}}"#,
            long_string,
            elements.join(",")
        );
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let err = pre_validate(&json, &value).unwrap_err();
        // Should have at least 2 errors: string too long + array too large
        assert!(
            err.len() >= 2,
            "Expected at least 2 errors, got {}: {:?}",
            err.len(),
            err
        );
    }

    #[test]
    fn test_pre_validate_input_too_large() {
        // Create a JSON string just over 5 MB
        let padding = "x".repeat(MAX_INPUT_SIZE);
        let json = format!(r#"{{"data": "{}"}}"#, padding);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let err = pre_validate(&json, &value).unwrap_err();
        assert!(err.iter().any(|e| e.contains("input size")));
    }

    #[test]
    fn test_pre_validate_value_string_too_long() {
        let long_string = "x".repeat(MAX_STRING_LENGTH + 1);
        let value = serde_json::json!({"name": long_string});
        let err = pre_validate_value(&value).unwrap_err();
        assert!(err.iter().any(|e| e.contains("string length")));
    }

    #[test]
    fn test_pre_validate_value_valid() {
        let value = serde_json::json!({"name": "Test", "value": 42});
        assert!(pre_validate_value(&value).is_ok());
    }
}
