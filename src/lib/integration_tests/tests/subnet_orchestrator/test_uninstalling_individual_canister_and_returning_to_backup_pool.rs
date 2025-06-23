use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    common::types::known_principal::KnownPrincipalType,
    constant::{GLOBAL_SUPER_ADMIN_USER_ID_V1, TEST_BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE},
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_charlie_principal_id},
};

#[test]
fn test_uninstalling_individual_canister_and_returning_to_backup_pool() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let global_admin = Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID_V1).unwrap();

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

    for i in 0..120 {
        pocket_ic.tick();
    }

    let alice_yral_principal_id = get_mock_user_alice_principal_id();
    let alice_yral_canister_id = pocket_ic
        .update_call(
            subnet_orchestrator_canister_id,
            alice_yral_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let canister_id: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id
        })
        .unwrap()
        .unwrap();

    pocket_ic
        .update_call(
            subnet_orchestrator_canister_id,
            global_admin,
            "uninstall_individual_user_canister",
            candid::encode_one(alice_yral_canister_id).unwrap(),
        )
        .map(|res| match res {
            WasmResult::Reply(payload) => {
                let result: Result<(), String> = candid::decode_one(&payload).unwrap();
                result
            }
            WasmResult::Reject(e) => panic!("Canister call failed {e}"),
        })
        .unwrap()
        .unwrap();

    let alice_canister_status = pocket_ic
        .canister_status(
            alice_yral_canister_id,
            Some(subnet_orchestrator_canister_id),
        )
        .unwrap();

    assert_eq!(alice_canister_status.module_hash, None);

    let user_canister_list = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            charlie_global_admin,
            "get_user_canister_list",
            candid::encode_one(()).unwrap(),
        )
        .map(|wasm_result| {
            let canister_list: Vec<Principal> = match wasm_result {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_list
        })
        .unwrap();

    assert!(!user_canister_list.contains(&alice_yral_canister_id));

    let backup_capacity = pocket_ic
        .query_call(
            subnet_orchestrator_canister_id,
            charlie_global_admin,
            "get_subnet_backup_capacity",
            candid::encode_one(()).unwrap(),
        )
        .map(|wasm_result| {
            let backup_capacity: u64 = match wasm_result {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            backup_capacity
        })
        .unwrap();

    assert_eq!(
        backup_capacity,
        TEST_BACKUP_INDIVIDUAL_USER_CANISTER_BATCH_SIZE + 1
    );
}
