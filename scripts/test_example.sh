#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

echo "Building wheelc (release)..."
cargo build --release

WC=target/release/wheelc
if [ ! -x "$WC" ]; then
  echo "wheelc not found at $WC"
  exit 1
fi

echo "Generating executable 'hello' from examples/hello.wheel"
"$WC" examples/hello.wheel -o hello --mode ge

echo "Running ./hello"
./hello || true

echo "Generating flat binary 'hello.bin'"
"$WC" examples/hello.wheel -o hello.bin --mode gb

echo "Artifacts:"
ls -l hello hello.bin || true

echo "done"
