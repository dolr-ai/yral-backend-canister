#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf "Usage: \n[-h Display help] \n";
  printf "This script builds and installs the rate_limits canister.\n";
  printf "Prerequisites: \n";
  printf "  - dfx must be running\n";
  printf "  - Rust toolchain with wasm32-unknown-unknown target\n";
  printf "  - candid-extractor tool (optional, for candid generation)\n";
  printf "\nThis script will automatically:\n";
  printf "  - Build the rate_limits canister WASM\n";
  printf "  - Generate Candid interface\n";
  printf "  - Compress the WASM file\n";
  printf "  - Create and install the canister\n";
  exit 0;
}

while getopts "h" arg; do
  case $arg in
    h)
      usage
      ;;
  esac
done

# Check if dfx is running
if ! dfx ping > /dev/null 2>&1; then
    echo "Error: dfx is not running. Please start dfx with 'dfx start'"
    exit 1
fi


echo "Creating rate_limits canister..."
dfx canister create --no-wallet rate_limits

# Build and generate candid for rate_limits
echo "Building rate_limits canister..."
dfx build rate_limits

# Generate candid for rate_limits
echo "Generating Candid interface for rate_limits..."
candid-extractor target/wasm32-unknown-unknown/release/rate_limits.wasm > src/canister/rate_limits/can.did || {
    echo "Warning: Failed to generate candid with candid-extractor"
    echo "Using existing Candid file or auto-generated version"
}

# Gzip the WASM file
WASM_PATH="./target/wasm32-unknown-unknown/release/rate_limits.wasm"
GZIPPED_WASM_PATH="./target/wasm32-unknown-unknown/release/rate_limits.wasm.gz"

if [ -f "$WASM_PATH" ]; then
    echo "Compressing WASM file..."
    gzip -f -1 "$WASM_PATH"
else
    echo "Error: WASM file not found at $WASM_PATH"
    exit 1
fi

# Check if gzipped WASM exists
if [ ! -f "$GZIPPED_WASM_PATH" ]; then
    echo "Error: Failed to create gzipped WASM at $GZIPPED_WASM_PATH"
    exit 1
fi

echo "Installing rate_limits canister..."
dfx canister install rate_limits --argument "(record {
  version = \"v1.0.0\"
})"

echo "Successfully installed rate_limits canister!"
echo "Canister ID: $(dfx canister id rate_limits)"
echo "WASM size: $(ls -lh $GZIPPED_WASM_PATH | awk '{print $5}')"
echo "Candid interface: src/canister/rate_limits/can.did"