# !/bin/bash


dfx build notification_store --network=ic

dfx canister stop notification_store --network=ic

dfx canister install notification_store --mode=upgrade --network=ic --argument "(record {
  version= \"v2.2.0\"
})"


dfx canister start notification_store --network=ic