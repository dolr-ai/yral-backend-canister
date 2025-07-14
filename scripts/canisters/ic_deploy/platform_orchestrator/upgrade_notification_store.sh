# !/bin/bash


dfx build notification_store --network=ic

dfx canister stop notification_store --network=ic

dfx canister install notification_store --mode=upgrade --network=ic 


dfx canister start notification_store --network=ic