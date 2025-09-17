#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf "Usage: \n[-h Display help] \n";
  printf "This script builds and installs the user_info_service canister.\n";
  printf "Prerequisites: \n";
  printf "  - dfx must be running\n";
  printf "  - Rust toolchain with wasm32-unknown-unknown target\n";
  printf "  - candid-extractor tool (optional, for candid generation)\n";
  printf "\nThis script will automatically:\n";
  printf "  - Build the user_info_service canister WASM\n";
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

echo "Creating user_info_service canister..."
dfx canister create --no-wallet user_info_service

# Build and generate candid for user_info_service
echo "Building user_info_service canister..."
dfx build user_info_service

# Generate candid for user_info_service
echo "Generating Candid interface for user_info_service..."
candid-extractor target/wasm32-unknown-unknown/release/user_info_service.wasm > src/canister/user_info_service/can.did || {
    echo "Warning: Failed to generate candid with candid-extractor"
    echo "Using existing Candid file or auto-generated version"
}

# Gzip the WASM file
WASM_PATH="./target/wasm32-unknown-unknown/release/user_info_service.wasm"
GZIPPED_WASM_PATH="./target/wasm32-unknown-unknown/release/user_info_service.wasm.gz"

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

echo "Installing user_info_service canister..."
dfx canister install user_info_service --argument "(record {
  version = \"v1.0.0\"
})" --mode reinstall

echo "Successfully installed user_info_service canister!"
echo "Canister ID: $(dfx canister id user_info_service)"
echo "WASM size: $(ls -lh $GZIPPED_WASM_PATH | awk '{print $5}')"
echo "Candid interface: src/canister/user_info_service/can.did"