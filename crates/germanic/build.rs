//! # Build script for GERMANIC
//!
//! Two responsibilities:
//! 1. Copy practice schema from source-of-truth into crate directory
//!    so that `include_str!` works for both `cargo build` and `cargo publish`.
//! 2. FlatBuffer bindings are pre-generated (no-op, see ADR-009).
//!
//! ## Regenerating FlatBuffers after schema changes
//!
//! If you modify `.fbs` files, run:
//! ```sh
//! ./scripts/regenerate-flatbuffers.sh
//! ```
//! This requires `flatc` (brew install flatbuffers) and will update
//! the pre-generated files in-place.

use std::fs;
use std::path::Path;

fn main() {
    copy_practice_schema();
}

/// Copy the practice schema definition from the workspace-level schemas/
/// directory into crates/germanic/schemas/ so that include_str!() can
/// reference it during both local builds and crates.io publish.
///
/// Source of truth: schemas/definitions/de/de.gesundheit.praxis.v1.schema.json
/// Copy target:     crates/germanic/schemas/de.gesundheit.praxis.v1.schema.json
fn copy_practice_schema() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest = Path::new(&manifest_dir);

    // Source: workspace root → schemas/definitions/de/
    let source = manifest.join("../../schemas/definitions/de/de.gesundheit.praxis.v1.schema.json");

    // Target: crate-local schemas/
    let target_dir = manifest.join("schemas");
    let target = target_dir.join("de.gesundheit.praxis.v1.schema.json");

    // Only copy if source exists (it won't exist during cargo publish
    // from the crates.io tarball — the target is already included)
    if source.exists() {
        fs::create_dir_all(&target_dir).expect("Failed to create schemas/ dir");
        fs::copy(&source, &target).expect("Failed to copy practice schema");

        // Tell Cargo to re-run if the source schema changes
        println!("cargo::rerun-if-changed={}", source.display());
    }
}
