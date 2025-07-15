#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf "Usage: \n[-s Skip test] \n[-h Display help] \n";
  exit 0;
}

skip_test=false

while getopts "sih" arg; do
  case $arg in
    s)
      skip_test=true
      ;;
    h)
      usage
      ;;
  esac
done

scripts/canisters/local_deploy/setup_icp_ledger.sh
scripts/canisters/local_deploy/setup_dollr_ledger.sh
scripts/canisters/local_deploy/create_sns_wasm.sh

dfx canister create --no-wallet individual_user_template
dfx canister create --no-wallet user_index
dfx canister create --no-wallet platform_orchestrator
dfx canister create --no-wallet notification_store
dfx canister create --no-wallet user_info_service

gzip_canister() {
  gzip -f -1 ./target/wasm32-unknown-unknown/release/$1.wasm
}

scripts/candid_generator.sh

gzip_canister individual_user_template
gzip_canister user_index
gzip_canister platform_orchestrator
gzip_canister notification_store
gzip_canister user_info_service

if [[ $skip_test != true ]]
then
  cargo test
fi


dfx canister install platform_orchestrator --argument "(record {
  version = \"v1.0.0\"
})"

dfx canister install user_info_service --argument "(record {
  version = \"v1.0.0\"
})"


dfx canister install user_index --argument "(record {
  known_principal_ids = opt vec {
    record {
      variant { CanisterIdPlatformOrchestrator };
      principal \"$(dfx canister id platform_orchestrator)\";
    };
    record {
      variant { UserIdGlobalSuperAdmin };
      principal \"$(dfx identity get-principal)\";
    };
    record {
      variant { CanisterIdUserIndex };
      principal \"$(dfx canister id user_index)\";
    };
    record {
      variant { CanisterIdSnsWasm };
      principal \"$(dfx canister id sns_wasm)\";
    };
    record {
      variant { CanisterIdSnsLedger };
      principal \"$(dfx canister id dollr_mock_ledger)\";
    };
  };
  access_control_map = opt vec {
    record {
      principal \"$(dfx identity get-principal)\";
      vec { variant { CanisterAdmin }; variant { CanisterController }; }
    };
  };
  version= \"v1.0.0\"
})"

dfx canister install notification_store --argument "(record {
  version= \"v1.0.0\"
})"

scripts/canisters/local_deploy/create_pool_of_individual_canister_user_index.sh

dfx canister update-settings user_index --set-controller $(dfx canister id platform_orchestrator) --yes 
dfx canister call platform_orchestrator register_new_subnet_orchestrator  "(principal \"$(dfx canister id user_index)\", true)"

dfx canister call dollr_mock_ledger "icrc1_transfer" "(record {
  to = record {
    owner = principal \"$(dfx canister id user_index)\";
    subaccount = null;
  };
  fee = null;
  memo = null;
  from_subaccount = null;
  created_at_time = null;
  amount = 9999990000;
})"
