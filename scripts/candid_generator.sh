#!/usr/bin/env bash

function generate_did() {
  local canister=$1
  canister_root="src/canister/$canister"


  candid-extractor "target/wasm32-unknown-unknown/release/$canister.wasm" > "$canister_root/can.did"
}

# The list of canisters of your project
CANISTERS=user_index,platform_orchestrator,individual_user_template,user_info_service,notification_store

for canister in $(echo $CANISTERS | sed "s/,/ /g")
do
    dfx build $canister
    generate_did "$canister"
done
