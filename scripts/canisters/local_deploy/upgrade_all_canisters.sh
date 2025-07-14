#!/usr/bin/env bash
set -euo pipefail

scripts/candid_generator.sh

usage() {
  printf "Usage: \n[-s Skip test] \n[-h Display help] \n";
  exit 0;
}

skip_test=false

while getopts "sh" arg; do
  case $arg in
    s)
      skip_test=true
      ;;
    h)
      usage
      ;;
  esac
done

dfx build platform_orchestrator
gzip -f -1 ./target/wasm32-unknown-unknown/release/platform_orchestrator.wasm

if [[ $skip_test != true ]]
then
  cargo test
fi

dfx canister install platform_orchestrator --mode upgrade --argument "(record {
  version= \"v2.2.0\"
})"

dfx canister install notification_store --mode upgrade

scripts/canisters/local_deploy/upgrade_subnet_orchestrator.sh

sleep 2

scripts/canisters/local_deploy/start_upgrades_for_individual_canisters.sh
