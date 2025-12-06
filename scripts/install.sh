#!/usr/bin/env bash
set -euo pipefail

# Simple installer for wheelc (MVP)
BIN_NAME=wheelc
TARGET=/usr/local/bin/$BIN_NAME

if [ ! -f target/release/$BIN_NAME ]; then
  echo "Please build the project first: cargo build --release"
  exit 1
fi

sudo cp target/release/$BIN_NAME $TARGET
sudo chmod +x $TARGET

echo "Installed $BIN_NAME to $TARGET"
echo "If /usr/local/bin is not in your PATH, add:"
echo '  export PATH="/usr/local/bin:$PATH"'
