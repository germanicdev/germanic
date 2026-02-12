//! # Build script for GERMANIC
//!
//! Previously, this script compiled FlatBuffer schemas (.fbs → .rs) at
//! build time using `flatc`. This required every user to have `flatc`
//! installed, causing `cargo install germanic` to fail with cryptic errors.
//!
//! ## Current approach (Option C)
//!
//! Pre-generated `.rs` files are checked into the repository under
//! `crates/germanic/src/generated/`. The build script is now a no-op
//! for normal builds.
//!
//! ## Regenerating after schema changes
//!
//! If you modify `.fbs` files, run:
//! ```sh
//! ./scripts/regenerate-flatbuffers.sh
//! ```
//! This requires `flatc` (brew install flatbuffers) and will update
//! the pre-generated files in-place.

fn main() {
    // No-op: FlatBuffer bindings are pre-generated.
    // See scripts/regenerate-flatbuffers.sh for regeneration.
}
