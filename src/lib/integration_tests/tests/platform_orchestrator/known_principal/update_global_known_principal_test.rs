use candid::{encode_one, CandidType, Principal};
use ic_ledger_types::{BlockIndex, Tokens};
use pocket_ic::WasmResult;
use serde::Serialize;
use shared_utils::{
    canister_specific::
        user_index::types::BroadcastCallStatus
    ,
    common::
        types::
            known_principal::KnownPrincipalType
        
    ,
    constant::GOVERNANCE_CANISTER_ID,
};
use std::collections::{HashMap, HashSet};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    },
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

#[test]
fn when_global_known_principal_is_updated_it_is_reflected_in_all_canisters() {
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

    let first_subnet_orchestrator_canister_id: Principal = pocket_ic
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

    let second_subnet_orchestrator_canister_id: Principal = pocket_ic
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

    for _ in 0..30 {
        pocket_ic.tick();
    }

    let governance_canister_id = Principal::from_text(GOVERNANCE_CANISTER_ID).unwrap();

    //get alice canister-id
    let alice_principal_id = get_mock_user_alice_principal_id();
    let alice_cannister_id: Principal = pocket_ic.update_call(first_subnet_orchestrator_canister_id, alice_principal_id, "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer", candid::encode_one(()).unwrap())
    .map(|res| {
        let canister_id: Principal = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("Canister call failed")
        };
        canister_id
    })
    .unwrap();

    let bob_principal_id = get_mock_user_bob_principal_id();
    let bob_canister_id: Principal = pocket_ic.update_call(second_subnet_orchestrator_canister_id, bob_principal_id, "get_requester_principals_canister_id_create_if_not_exists_and_optionally_allow_referrer", candid::encode_one(()).unwrap())
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
            "update_global_known_principal",
            candid::encode_args((
                KnownPrincipalType::CanisterIdSnsGovernance,
                governance_canister_id,
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

    for _ in 0..20 {
        pocket_ic.tick();
    }

    let governance_canister_id_from_first_subnet = pocket_ic
        .query_call(
            first_subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::CanisterIdSnsGovernance).unwrap(),
        )
        .map(|res| {
            let post_cache_id: Option<Principal> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("update subnet_known_principal"),
            };
            post_cache_id
        })
        .unwrap();

    assert_eq!(
        governance_canister_id_from_first_subnet,
        Some(governance_canister_id)
    );

    let governance_canister_id_from_second_subnet = pocket_ic
        .query_call(
            second_subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::CanisterIdSnsGovernance).unwrap(),
        )
        .map(|res| {
            let post_cache_id: Option<Principal> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("update subnet_known_principal"),
            };
            post_cache_id
        })
        .unwrap();

    assert_eq!(
        governance_canister_id_from_second_subnet,
        Some(governance_canister_id)
    );

    let broadcast_call_res = pocket_ic
        .query_call(
            first_subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_last_broadcast_call_status",
            encode_one(()).unwrap(),
        )
        .map(|res| {
            let res: BroadcastCallStatus = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("get_last_broadcast_call_status failed"),
            };
            res
        })
        .unwrap();

    assert!(broadcast_call_res
        .method_name
        .eq("update_well_known_principal"));

    assert!(broadcast_call_res.total_canisters > 0);
    assert!(broadcast_call_res.failed_canisters_count == 0);
    assert!(broadcast_call_res.successful_canisters_count > 0);

    let governance_canister_id_from_alice = pocket_ic
        .query_call(
            alice_cannister_id,
            Principal::anonymous(),
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::CanisterIdSnsGovernance).unwrap(),
        )
        .map(|res| {
            let post_cache_id: Option<Principal> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("update subnet_known_principal"),
            };
            post_cache_id
        })
        .unwrap();

    assert_eq!(
        governance_canister_id_from_alice,
        Some(governance_canister_id)
    );

    let governance_canister_id_from_bob = pocket_ic
        .query_call(
            bob_canister_id,
            Principal::anonymous(),
            "get_well_known_principal_value",
            candid::encode_one(KnownPrincipalType::CanisterIdSnsGovernance).unwrap(),
        )
        .map(|res| {
            let post_cache_id: Option<Principal> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("update subnet_known_principal"),
            };
            post_cache_id
        })
        .unwrap();

    assert_eq!(
        governance_canister_id_from_bob,
        Some(governance_canister_id)
    );
}
