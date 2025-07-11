use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::common::types::known_principal::KnownPrincipalType;
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
         get_mock_user_alice_principal_id,
        get_mock_user_charlie_canister_id,
    },
};

#[test]
pub fn platform_orchestrator_on_receiving_call_from_global_admin_can_make_individual_canister_logs_public_test(
) {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let platform_orchestrator_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .copied()
        .unwrap();

    let super_admin = known_principal_map
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .copied()
        .unwrap();

    let charlie_global_admin_principal = get_mock_user_charlie_canister_id();

    pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            super_admin,
            "add_principal_as_global_admin",
            candid::encode_one(charlie_global_admin_principal).unwrap(),
        )
        .map(|wasm_result| {
            let res: () = match wasm_result {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n add_principal_as_global_admin call failedj"),
            };
            res
        })
        .unwrap();

    let subnet_orchestrator_canister_id = pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            charlie_global_admin_principal,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[0]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for _ in 0..50 {
        pocket_ic.tick()
    }

    let alice_principal = get_mock_user_alice_principal_id();

    let alice_individual_canister_id = pocket_ic
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

    /********* Only global platform orchestrator admin can make the logs public/private *************/
    let result_for_making_logs_private = pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            alice_principal,
            "make_individual_canister_logs_private",
            candid::encode_one(alice_individual_canister_id).unwrap(),
        )
        .map(|wasm_result| {
            let res: Result<(), String> = match wasm_result {
                WasmResult::Reply(payload) => Ok(candid::decode_one(&payload).unwrap()),
                WasmResult::Reject(e) => Err(e),
            };

            res
        })
        .unwrap();

    assert!(result_for_making_logs_private.is_err());

    let result_for_making_logs_private = pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            charlie_global_admin_principal,
            "make_individual_canister_logs_private",
            candid::encode_one(alice_individual_canister_id).unwrap(),
        )
        .map(|wasm_result| {
            let res: Result<(), String> = match wasm_result {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(e) => {
                    panic!("\n call to make_individual_canister_logs_private failed")
                }
            };

            res
        })
        .unwrap();

    assert!(result_for_making_logs_private.is_ok());

    let result_for_making_logs_public = pocket_ic
        .update_call(
            platform_orchestrator_canister_id,
            charlie_global_admin_principal,
            "make_individual_canister_logs_public",
            candid::encode_one(alice_individual_canister_id).unwrap(),
        )
        .map(|wasm_result| {
            let res: Result<(), String> = match wasm_result {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                WasmResult::Reject(_) => {
                    panic!("\n call to make_individual_canister_logs_private failed")
                }
            };

            res
        })
        .unwrap();

    assert!(result_for_making_logs_public.is_ok());
}
