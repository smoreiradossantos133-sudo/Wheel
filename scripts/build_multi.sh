#!/usr/bin/env bash
set -euo pipefail

# Build wheelc for Linux, Windows and macOS (x86_64) into dist/
# This script tries to add rust targets and build with cargo. For cross-compilation
# to Windows/macOS you may need additional toolchains or to use `cross`.

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
DIST_DIR="$ROOT_DIR/dist"
mkdir -p "$DIST_DIR"

build_target() {
  local triple=$1
  local outdir="$DIST_DIR/$triple"
  mkdir -p "$outdir"
  echo "Building target $triple"
  if command -v rustup >/dev/null 2>&1; then
    rustup target add "$triple" || true
  fi

  # Try native cargo build for the target
  if cargo build --release --target "$triple"; then
    BIN="target/$triple/release/wheelc"
    if [ -f "$BIN" ]; then
      cp "$BIN" "$outdir/wheelc"
      echo "Built and copied to $outdir/wheelc"
      return 0
    fi
  fi

  # Fallback: try cross (requires cargo-cross)
  if command -v cross >/dev/null 2>&1; then
    cross build --release --target "$triple"
    BIN="target/$triple/release/wheelc"
    if [ -f "$BIN" ]; then
      cp "$BIN" "$outdir/wheelc"
      echo "Built with cross and copied to $outdir/wheelc"
      return 0
    fi
  fi

  echo "Failed to build target $triple; ensure required toolchains are installed." >&2
  return 1
}

echo "Building for Linux (x86_64-unknown-linux-gnu)"
build_target x86_64-unknown-linux-gnu

echo "Building for Windows (x86_64-pc-windows-gnu)"
build_target x86_64-pc-windows-gnu || echo "Windows build skipped or failed"

echo "Building for macOS (x86_64-apple-darwin)"
build_target x86_64-apple-darwin || echo "macOS build skipped or failed"

echo "Done. Artifacts in $DIST_DIR"
