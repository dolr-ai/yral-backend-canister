use candid::{CandidType, Principal};
use ic_ledger_types::{BlockIndex, Tokens};
use pocket_ic::WasmResult;
use serde::{Deserialize, Serialize};
use shared_utils::
    common::
        types::
            known_principal::KnownPrincipalType
        
    
;
use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::get_mock_user_alice_principal_id,
};

pub type CanisterId = Principal;

#[derive(CandidType, Serialize)]
pub enum ChangeIndexId {
    Unset,
    SetTo(Principal),
}

#[derive(CandidType, Serialize)]
pub struct Config {
    /// The maximum number of transactions
    /// returned by the [icrc3_get_transactions]
    /// endpoint
    pub max_transactions_per_request: u64,

    /// The principal of the index canister
    /// for this ledger
    pub index_id: Option<Principal>,
}
#[derive(CandidType, Serialize)]
pub struct UpgradeArgs {
    pub max_transactions_per_request: Option<u64>,
    pub change_index_id: Option<ChangeIndexId>,
}

#[derive(CandidType, Serialize)]
enum LedgerArgs {
    Init(Config),
    Upgrade(Option<UpgradeArgs>),
}

#[derive(CandidType)]
struct NnsLedgerCanisterInitPayload {
    minting_account: String,
    initial_values: HashMap<String, Tokens>,
    send_whitelist: HashSet<CanisterId>,
    transfer_fee: Option<Tokens>,
}

// pub struct CyclesCanisterInitPayload {
//     pub ledger_canister_id: Option<CanisterId>,
//     pub governance_canister_id: Option<CanisterId>,
//     pub minting_account_id: Option<AccountIdentifier>,
//     pub last_purged_notification: Option<BlockIndex>,
//     pub exchange_rate_canister: Option<ExchangeRateCanister>,
//     pub cycles_ledger_canister_id: Option<CanisterId>,
// }

#[derive(CandidType)]
struct AuthorizedSubnetWorks {
    who: Option<Principal>,
    subnets: Vec<Principal>,
}

#[derive(CandidType)]
struct CyclesMintingCanisterInitPayload {
    ledger_canister_id: CanisterId,
    governance_canister_id: CanisterId,
    minting_account_id: Option<String>,
    last_purged_notification: Option<BlockIndex>,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct UpgradeStatus {
    pub version_number: u64,
    pub last_run_on: SystemTime,
    pub successful_upgrade_count: u32,
    pub failed_canister_ids: Vec<(Principal, Principal, String)>,
    #[serde(default)]
    pub version: String,
}

#[test]
fn when_subnet_known_principal_is_updated_it_is_reflected_in_individual_canisters() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let subnet_orchestrator_canister_id: Principal = pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[1]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for i in 0..50 {
        pocket_ic.tick();
    }

    let user_id_global_admin = Principal::anonymous();

    //get alice canister-id
    let alice_principal_id = get_mock_user_alice_principal_id();
    let alice_cannister_id: Principal = pocket_ic.update_call(subnet_orchestrator_canister_id, alice_principal_id, "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer", candid::encode_one(()).unwrap())
    .map(|res| {
        let canister_id: Principal = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("Canister call failed")
        };
        canister_id
    })
    .unwrap();

    //update subnet known principal
    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "update_subnet_known_principal",
            candid::encode_args((
                subnet_orchestrator_canister_id,
                KnownPrincipalType::UserIdGlobalSuperAdmin,
                user_id_global_admin,
            ))
            .unwrap(),
        )
        .map(|res| {
            let update_res: Result<String, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("update subnet_known_principal"),
            };
            update_res
        })
        .unwrap()
        .unwrap();

    let super_admin = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::UserIdGlobalSuperAdmin).unwrap(),
        )
        .map(|res| {
            let post_cache_id: Option<Principal> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get subnet_known_principal"),
            };
            post_cache_id
        })
        .unwrap();

    assert_eq!(super_admin, Some(Principal::anonymous()));

    let super_admin = pocket_ic
        .query_call(
            alice_cannister_id,
            Principal::anonymous(),
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::UserIdGlobalSuperAdmin).unwrap(),
        )
        .map(|res| {
            let post_cache_id: Option<Principal> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get individual_known_principal"),
            };
            post_cache_id
        })
        .unwrap();

    assert_eq!(super_admin, Some(Principal::anonymous()))
}
