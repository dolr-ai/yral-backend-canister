use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::session::SessionType,
    common::types::known_principal::KnownPrincipalType, constant::GLOBAL_SUPER_ADMIN_USER_ID,
};
use test_utils::setup::{
    env::{pocket_ic_env::get_new_pocket_ic_env, pocket_ic_init::get_initialized_env_with_provisioned_known_canisters},
    test_constants::get_mock_user_alice_principal_id,
};

#[test]
fn update_session_type_tests() {
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

    let session_type = pocket_ic
        .query_call(
            alice_canister_id,
            alice_principal_id,
            "get_session_type",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let session_type_res: Result<SessionType, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_session_type failed\n"),
            };
            session_type_res
        })
        .unwrap()
        .unwrap();
    assert_eq!(session_type, SessionType::AnonymousSession);

    // user cannot update the session type
    pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "update_session_type",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .unwrap();

    let session_type = pocket_ic
        .query_call(
            alice_canister_id,
            alice_principal_id,
            "get_session_type",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let session_type_res: Result<SessionType, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_session_type failed\n"),
            };
            session_type_res
        })
        .unwrap()
        .unwrap();

    assert_eq!(session_type, SessionType::AnonymousSession);

    // global admin can update the session type of canister
    pocket_ic
        .update_call(
            alice_canister_id,
            Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap(),
            "update_session_type",
            candid::encode_one(SessionType::RegisteredSession).unwrap(),
        )
        .unwrap();

    let session_type = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_session_type",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let session_type_res: Result<SessionType, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_session_type failed\n"),
            };
            session_type_res
        })
        .unwrap()
        .unwrap();

    assert_eq!(session_type, SessionType::RegisteredSession);

    //once set to registered session cannot update back to Anonymous Session
    pocket_ic
        .update_call(
            alice_canister_id,
            Principal::from_text(GLOBAL_SUPER_ADMIN_USER_ID).unwrap(),
            "update_session_type",
            candid::encode_one(SessionType::AnonymousSession).unwrap(),
        )
        .unwrap();

    let session_type = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_session_type",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let session_type_res: Result<SessionType, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_session_type failed\n"),
            };
            session_type_res
        })
        .unwrap()
        .unwrap();

    assert_eq!(session_type, SessionType::RegisteredSession)
}