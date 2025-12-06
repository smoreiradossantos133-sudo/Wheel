#!/usr/bin/env bash
set -euo pipefail

# Install wheel binaries from dist/ into a system root and add to PATH
# Usage: ./install_wheel.sh [dist-dir] [install-root]

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
DIST_DIR=${1:-"$ROOT_DIR/dist"}
INSTALL_ROOT=${2:-"/usr/local/wheel"}

echo "Installing Wheel from $DIST_DIR to $INSTALL_ROOT"
sudo mkdir -p "$INSTALL_ROOT/bin"
sudo chown root:root "$INSTALL_ROOT" || true

for bin in "$DIST_DIR"/*/wheelc; do
  if [ -f "$bin" ]; then
    echo "Installing $bin"
    sudo cp "$bin" "$INSTALL_ROOT/bin/"
    sudo chmod a+rx "$INSTALL_ROOT/bin/$(basename $bin)"
  fi
done

# Add INSTALL_ROOT/bin to /etc/profile.d/wheel_path.sh if not present
PROFILE_D="/etc/profile.d/wheel_path.sh"
echo "Adding $INSTALL_ROOT/bin to system PATH via $PROFILE_D"
sudo bash -c "cat > $PROFILE_D <<'EOF'
# Added by Wheel installer
export PATH=\"$INSTALL_ROOT/bin:$PATH\"
EOF"

echo "Installation complete. You may need to log out and log in for PATH changes to apply."
