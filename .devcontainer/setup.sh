#!/bin/bash
set -e

echo "Starting dfx installation..."

# Install dfx non-interactively
export DFXVM_INIT_YES=true
DFX_VERSION=0.31.0 sh -c "$(curl -fsSL https://internetcomputer.org/install.sh)"

# Add dfx to PATH for future sessions
echo 'export PATH="$HOME/.local/share/dfx/bin:$PATH"' >> ~/.bashrc

echo "dfx installation completed"
