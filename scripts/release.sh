#!/usr/bin/env bash
set -euo pipefail

# Build release and package into a zip with http/ and lua/ directories

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

# Extract package info from Cargo.toml
PKG_NAME=$(grep '^name' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
PKG_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

echo "Building $PKG_NAME v$PKG_VERSION in release mode..."
cargo build --release

TARGET_DIR="$PROJECT_DIR/target/release"
EXE_NAME="$PKG_NAME"

# Check platform for executable extension
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    EXE_NAME="${PKG_NAME}.exe"
fi

EXE_PATH="$TARGET_DIR/$EXE_NAME"

if [[ ! -f "$EXE_PATH" ]]; then
    echo "ERROR: Executable not found at $EXE_PATH"
    exit 1
fi

ZIP_NAME="${PKG_NAME}-${PKG_VERSION}.zip"
ZIP_PATH="$TARGET_DIR/$ZIP_NAME"

# Create a staging directory for clean zip structure
STAGING_DIR=$(mktemp -d)
trap 'rm -rf "$STAGING_DIR"' EXIT

cp "$EXE_PATH" "$STAGING_DIR/"
cp -r "$PROJECT_DIR/http" "$STAGING_DIR/"
cp -r "$PROJECT_DIR/lua" "$STAGING_DIR/"

# Create zip
rm -f "$ZIP_PATH"
(cd "$STAGING_DIR" && zip -r "$ZIP_PATH" .)

echo "Release zip created: $ZIP_PATH"
