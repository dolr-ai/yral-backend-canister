use candid::Principal;
use hex::ToHex;
use ic_cdk::println;
use pocket_ic::WasmResult;
use shared_utils::{
    common::types::known_principal::KnownPrincipalType,
    constant::{SNS_TOKEN_GOVERNANCE_MODULE_HASH, SNS_WASM_W_PRINCIPAL_ID},
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
         get_mock_user_alice_principal_id,
        get_mock_user_charlie_principal_id,
    },
};

use crate::utils::{setup_default_sns_creator_token, setup_sns_w_canister_for_creator_dao};

#[test]
pub fn test_skip_upgrading_creator_sns_governance_canister_if_version_is_present() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = *known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let charlie_global_admin = get_mock_user_charlie_principal_id();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "add_principal_as_global_admin",
            candid::encode_one(charlie_global_admin).unwrap(),
        )
        .unwrap();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "update_global_known_principal",
            candid::encode_args((
                KnownPrincipalType::CanisterIdSnsWasm,
                Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap(),
            ))
            .unwrap(),
        )
        .unwrap();

    let subnet_orchestrator_canister_id: Principal = pocket_ic
        .update_call(
            platform_canister_id,
            charlie_global_admin,
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

    let alice_principal = get_mock_user_alice_principal_id();
    let alice_canister_id: Principal = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id,
            alice_principal,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();

    let alice_initial_cycle_balance = pocket_ic.cycle_balance(alice_canister_id);
    let sns_wasm_w_canister_id = Principal::from_text(SNS_WASM_W_PRINCIPAL_ID).unwrap();

    setup_sns_w_canister_for_creator_dao(&pocket_ic, super_admin);

    let deployed_canister = setup_default_sns_creator_token(
        &pocket_ic,
        super_admin,
        alice_principal,
        alice_canister_id,
    );
    let custom_governance_wasm =
        include_bytes!("../../../../../wasms/custom-governance-canister.wasm.gz");

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "upgrade_all_creator_dao_governance_canisters_in_the_network",
            candid::encode_one(custom_governance_wasm).unwrap(),
        )
        .unwrap();

    for _ in 0..110 {
        pocket_ic.tick();
    }

    let governance_canister_status = pocket_ic
        .canister_status(deployed_canister.governance, Some(alice_canister_id))
        .unwrap();

    let governance_canister_module_hash = governance_canister_status
        .module_hash
        .unwrap()
        .encode_hex::<String>();

    println!(
        "Custom governance canister module hash {}",
        &governance_canister_module_hash
    );

    assert!(governance_canister_module_hash.eq(&SNS_TOKEN_GOVERNANCE_MODULE_HASH.to_owned()));
}
