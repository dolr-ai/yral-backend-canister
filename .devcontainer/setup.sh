#!/bin/bash
set -e

echo "Starting dfx installation..."

# Install dfx non-interactively with timeout
export DFXVM_INIT_YES=true
timeout 300 sh -c "DFX_VERSION=0.24.3 sh -c \"\$(curl -fsSL https://internetcomputer.org/install.sh)\"" || {
    echo "dfx installation timed out or failed, continuing..."
    exit 0
}

# Source dfx environment for current session if it exists
if [ -f "$HOME/.local/share/dfx/env" ]; then
    source "$HOME/.local/share/dfx/env"
    echo "dfx installed successfully"
else
    echo "dfx installation may have failed, but continuing..."
fi
