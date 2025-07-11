

use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{canister_specific::individual_user_template::types::session::SessionType, common::types::known_principal::KnownPrincipalType};
use test_utils::setup::{
    env::{pocket_ic_env::get_new_pocket_ic_env, pocket_ic_init::get_initialized_env_with_provisioned_known_canisters},
    test_constants::get_mock_user_alice_principal_id,
};

#[test]
fn update_last_access_time_test() {
    let (pocket_ic, known_principal_map) = get_new_pocket_ic_env();
    let known_principal_map = get_initialized_env_with_provisioned_known_canisters(&pocket_ic, known_principal_map);
    let user_index_canister_id = known_principal_map
        .get(&KnownPrincipalType::CanisterIdUserIndex)
        .unwrap();
    let alice_principal_id = get_mock_user_alice_principal_id();

    let alice_canister_id = pocket_ic
        .update_call(
            *user_index_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let alice_canister_id: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!(
                    "\n🛑 get_requester_principals_canister_id_create_if_not_exists failed\n"
                ),
            };
            alice_canister_id
        })
        .unwrap()
        .unwrap();

    let update_last_access_time_result = pocket_ic
        .update_call(
            alice_canister_id,
            Principal::anonymous(),
            "update_last_access_time",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .map(|reply_payload| {
            let update_last_access_time_res: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_last_access_time failed\n"),
            };
            update_last_access_time_res
        })
        .unwrap();

    assert!(update_last_access_time_result.is_err());

    let update_last_access_time_result = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "update_last_access_time",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .map(|reply_payload| {
            let update_last_access_time_res: Result<String, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 update_last_access_time failed\n"),
            };
            update_last_access_time_res
        })
        .unwrap();

    update_last_access_time_result.unwrap();
}