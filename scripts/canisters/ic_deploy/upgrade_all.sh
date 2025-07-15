#!/usr/bin/env bash
set -euo pipefail

dfx build --network=ic individual_user_template
gzip -f -1 ./target/wasm32-unknown-unknown/release/individual_user_template.wasm
dfx build --network=ic user_index
dfx build --network=ic notification_store

dfx canister install user_index --network ic --mode upgrade --argument "(record {})"
dfx canister install notification_store --network ic --mode upgrade