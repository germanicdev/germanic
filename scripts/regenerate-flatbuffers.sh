#!/usr/bin/env bash
# regenerate-flatbuffers.sh
#
# Regenerates pre-committed FlatBuffer Rust bindings from .fbs schemas.
# Run this after modifying any .fbs file in schemas/.
#
# Requirements: flatc (brew install flatbuffers)
#
# Usage:
#   ./scripts/regenerate-flatbuffers.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCHEMA_DIR="$REPO_ROOT/schemas"
OUT_DIR="$REPO_ROOT/crates/germanic/src/generated"
TEMP_DIR="$(mktemp -d)"

trap 'rm -rf "$TEMP_DIR"' EXIT

# =========================================================================
# Step 1: Verify flatc is available
# =========================================================================

if ! command -v flatc &>/dev/null; then
    echo "ERROR: flatc not found!" >&2
    echo "" >&2
    echo "Install the FlatBuffers compiler:" >&2
    echo "  macOS:  brew install flatbuffers" >&2
    echo "  Linux:  apt install flatbuffers-compiler" >&2
    echo "  Manual: https://github.com/google/flatbuffers/releases" >&2
    exit 1
fi

echo "Using $(flatc --version)"

# =========================================================================
# Step 2: Compile all .fbs schemas in one call
# =========================================================================
#
# IMPORTANT: All schemas in ONE call for correct namespace resolution.
# Order matters: base schemas (no dependencies) first!

SCHEMAS=(
    "common/meta.fbs"
    "de/praxis.fbs"
)

SCHEMA_PATHS=()
for s in "${SCHEMAS[@]}"; do
    SCHEMA_PATHS+=("$SCHEMA_DIR/$s")
done

echo "Compiling schemas: ${SCHEMAS[*]}"

flatc --rust \
    -o "$TEMP_DIR" \
    -I "$SCHEMA_DIR" \
    "${SCHEMA_PATHS[@]}"

echo "flatc generated:"
ls -la "$TEMP_DIR"/*_generated.rs

# =========================================================================
# Step 3: Post-processing — fix cross-namespace bug (#5275)
# =========================================================================
#
# flatc generates faulty relative paths like super::super::germanic::common::
# These only work if ALL namespaces are in ONE file. Since we have separate
# files, we replace with absolute crate:: paths.
#
# See: https://github.com/google/flatbuffers/issues/5275

echo "Fixing cross-namespace paths..."

for file in "$TEMP_DIR"/*_generated.rs; do
    # praxis_generated.rs references germanic.common.* from meta_generated.rs
    sed -i.bak \
        -e 's|super::super::super::germanic::common::|crate::generated::meta::germanic::common::|g' \
        -e 's|super::super::germanic::common::|crate::generated::meta::germanic::common::|g' \
        -e 's|super::super::de::gesundheit::|crate::generated::praxis::de::gesundheit::|g' \
        "$file"
    rm -f "$file.bak"
done

# =========================================================================
# Step 4: Copy to source tree
# =========================================================================

mkdir -p "$OUT_DIR"
cp "$TEMP_DIR"/meta_generated.rs "$OUT_DIR/meta_generated.rs"
cp "$TEMP_DIR"/praxis_generated.rs "$OUT_DIR/praxis_generated.rs"

echo ""
echo "Updated:"
echo "  $OUT_DIR/meta_generated.rs"
echo "  $OUT_DIR/praxis_generated.rs"
echo ""
echo "Don't forget to commit the updated files!"
