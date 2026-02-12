//! # Build script for GERMANIC
//!
//! Compiles FlatBuffer schemas to Rust code and fixes the known
//! cross-namespace bug (#5275) through post-processing.
//!
//! ## The Bug (open since 2019, will never be fixed)
//!
//! flatc generates faulty relative paths like `super::super::common::`.
//! These only work if ALL namespaces are in ONE file.
//!
//! ## Our Solution
//!
//! After code generation we replace the faulty paths with
//! correct absolute `crate::` paths. A simple string replace.
//!
//! See: https://github.com/google/flatbuffers/issues/5275

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // =========================================================================
    // CONFIGURATION
    // =========================================================================

    // Relative path to schemas (from crates/germanic/)
    let schema_dir = Path::new("../../schemas");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");

    // Schemas in DEPENDENCY ORDER (base first!)
    // CRITICAL: If schema A depends on schema B, B must come BEFORE A!
    let schemas = [
        "common/meta.fbs", // No dependencies - base
        "de/praxis.fbs",   // Could later reference meta.fbs
    ];

    // =========================================================================
    // STEP 1: Check flatc availability
    // =========================================================================

    let flatc_version = Command::new("flatc").arg("--version").output();

    match flatc_version {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("cargo:warning=GERMANIC: Using {}", version.trim());
        }
        Err(_) => {
            panic!(
                r#"
╔═══════════════════════════════════════════════════════════════╗
║  ERROR: flatc not found!                                      ║
║                                                               ║
║  Please install the FlatBuffers compiler:                     ║
║                                                               ║
║  macOS:   brew install flatbuffers                            ║
║  Linux:   apt install flatbuffers-compiler                    ║
║                                                               ║
║  Or download from: https://github.com/google/flatbuffers/releases║
╚═══════════════════════════════════════════════════════════════╝
"#
            );
        }
    }

    // =========================================================================
    // STEP 2: Compile FlatBuffers
    // =========================================================================
    //
    // IMPORTANT: Compile all schemas in ONE call!
    // This is crucial for correct namespace resolution.

    let schema_paths: Vec<_> = schemas.iter().map(|s| schema_dir.join(s)).collect();

    let mut cmd = Command::new("flatc");
    cmd.arg("--rust")
        .arg("-o")
        .arg(&out_dir)
        .arg("-I")
        .arg(schema_dir); // Include path for cross-references

    for path in &schema_paths {
        cmd.arg(path);
    }

    println!("cargo:warning=GERMANIC: Executing: {:?}", cmd);

    let status = cmd.status().expect("Could not execute flatc");

    if !status.success() {
        panic!(
            r#"
╔═══════════════════════════════════════════════════════════════╗
║  ERROR: flatc failed!                                         ║
║                                                               ║
║  Check your schema files with:                                ║
║  flatc --rust -I ../../schemas ../../schemas/de/praxis.fbs    ║
╚═══════════════════════════════════════════════════════════════╝
"#
        );
    }

    // =========================================================================
    // STEP 3: POST-PROCESSING - Fix cross-namespace bug
    // =========================================================================
    //
    // flatc generates:   super::super::germanic::common::...
    // We need:           crate::generated::germanic::common::...
    //
    // The fix is a simple string replace in the generated code.

    fix_cross_namespace_paths(&out_dir);

    // =========================================================================
    // STEP 4: Set rebuild triggers
    // =========================================================================

    for schema in &schemas {
        println!("cargo:rerun-if-changed=../../schemas/{}", schema);
    }
    println!("cargo:rerun-if-changed=build.rs");

    // Debug info: Show what was generated
    show_generated_files(&out_dir);

    println!(
        "cargo:warning=GERMANIC: FlatBuffer code generated in {}",
        out_dir
    );
}

/// Fixes the faulty cross-namespace paths in generated code.
///
/// # The Bug (#5275)
///
/// flatc's Rust codegen generates `super::super::...` paths for
/// cross-namespace references. These only work if ALL
/// namespaces are in ONE file.
///
/// Since we have separate files, we must replace the relative paths
/// with absolute `crate::` paths.
fn fix_cross_namespace_paths(out_dir: &str) {
    // =========================================================================
    // PATH MAPPINGS
    // =========================================================================
    //
    // Format: (what flatc generates, what we need)
    //
    // IMPORTANT: These mappings must be adjusted when:
    // - New schemas with cross-namespace references are added
    // - The module structure in lib.rs is changed
    //
    // Order may matter - more specific patterns first!

    let mappings = [
        // ─────────────────────────────────────────────────────────────────────
        // praxis.fbs (in crate::generated::praxis) references
        // germanic.common.* from meta.fbs (in crate::generated::meta)
        // ─────────────────────────────────────────────────────────────────────
        //
        // flatc generates:    super::super::germanic::common::GermanicMeta
        // We need:            crate::generated::meta::germanic::common::GermanicMeta
        //
        (
            "super::super::germanic::common::",
            "crate::generated::meta::germanic::common::",
        ),
        // Fallback for deeper namespace hierarchies
        (
            "super::super::super::germanic::common::",
            "crate::generated::meta::germanic::common::",
        ),
        // ─────────────────────────────────────────────────────────────────────
        // If meta.fbs ever references praxis.fbs (unlikely)
        // ─────────────────────────────────────────────────────────────────────
        (
            "super::super::de::gesundheit::",
            "crate::generated::praxis::de::gesundheit::",
        ),
    ];

    // Find all generated files
    let generated_files = find_generated_files(out_dir);

    for file_path in generated_files {
        if let Ok(content) = fs::read_to_string(&file_path) {
            let mut fixed = content.clone();
            let mut changes = 0;

            for (old, new) in &mappings {
                if fixed.contains(*old) {
                    let count = fixed.matches(*old).count();
                    fixed = fixed.replace(*old, new);
                    changes += count;

                    println!(
                        "cargo:warning=GERMANIC: {} replacements in {}: {} → {}",
                        count,
                        file_path.file_name().unwrap_or_default().to_string_lossy(),
                        old,
                        new
                    );
                }
            }

            // Only write if something changed
            if fixed != content {
                fs::write(&file_path, &fixed).expect("Could not write fixed file");

                println!(
                    "cargo:warning=GERMANIC: {} cross-namespace paths fixed in {}",
                    changes,
                    file_path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
        }
    }
}

/// Finds all *_generated.rs files in OUT_DIR (recursively).
fn find_generated_files(out_dir: &str) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    fn recursive(path: &Path, files: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    recursive(&path, files);
                } else if path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with("_generated.rs"))
                {
                    files.push(path);
                }
            }
        }
    }

    recursive(Path::new(out_dir), &mut files);
    files
}

/// Shows the generated files for debugging purposes.
fn show_generated_files(out_dir: &str) {
    let files = find_generated_files(out_dir);

    if files.is_empty() {
        println!("cargo:warning=GERMANIC: No *_generated.rs files found!");
    } else {
        println!(
            "cargo:warning=GERMANIC: {} generated files:",
            files.len()
        );
        for file in &files {
            // Relative path for better readability
            let relative = file
                .strip_prefix(out_dir)
                .unwrap_or(file)
                .display();
            println!("cargo:warning=GERMANIC:   - {}", relative);
        }
    }
}
