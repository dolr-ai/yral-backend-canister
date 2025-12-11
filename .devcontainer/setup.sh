#!/bin/bash
set -e

# Install dfx non-interactively
export DFXVM_INIT_YES=true
DFX_VERSION=0.24.3 sh -c "$(curl -fsSL https://internetcomputer.org/install.sh)"

# Source dfx environment for current session
source "$HOME/.local/share/dfx/env"
