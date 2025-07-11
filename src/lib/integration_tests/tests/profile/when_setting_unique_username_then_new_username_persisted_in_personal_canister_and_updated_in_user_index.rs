use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontend,
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::{pocket_ic_env::get_new_pocket_ic_env, pocket_ic_init::get_initialized_env_with_provisioned_known_canisters},
    test_constants::get_mock_user_alice_principal_id,
};

#[ignore]
#[test]
fn when_setting_unique_username_then_new_username_persisted_in_personal_canister_and_updated_in_user_index() {
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

    pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "update_profile_set_unique_username_once",
            candid::encode_one(String::from("cool_alice_1234")).unwrap(),
        )
        .unwrap();

    let profile_details_from_user_canister = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_profile_details",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile_details_from_user_canister: UserProfileDetailsForFrontend =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile_details failed\n"),
                };
            profile_details_from_user_canister
        })
        .unwrap();

    assert_eq!(
        profile_details_from_user_canister.unique_user_name,
        Some("cool_alice_1234".to_string())
    );

    let is_alice_username_taken = pocket_ic
        .query_call(
            *user_index_canister_id,
            Principal::anonymous(),
            "get_index_details_is_user_name_taken",
            candid::encode_one("cool_alice_1234").unwrap(),
        )
        .map(|reply_payload| {
            let is_alice_username_taken: bool = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_index_details_is_user_name_taken failed\n"),
            };
            is_alice_username_taken
        })
        .unwrap();

    assert!(is_alice_username_taken);

    println!(
        "🧪 profile_details_from_user_canister: {:?}",
        profile_details_from_user_canister
    );

    println!("🧪 is_alice_username_taken: {:?}", is_alice_username_taken);

    let alice_canister_id_corresponding_to_username = pocket_ic
        .query_call(
            *user_index_canister_id,
            Principal::anonymous(),
            "get_user_canister_id_from_unique_user_name",
            candid::encode_one("cool_alice_1234".to_string()).unwrap(),
        )
        .map(|reply_payload| {
            let alice_principal_id_corresponding_to_username: Option<Principal> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_user_canister_id_from_unique_user_name failed\n"),
                };
            alice_principal_id_corresponding_to_username
        })
        .unwrap();

    println!(
        "🧪 alice_canister_id_corresponding_to_username: {:?}",
        alice_canister_id_corresponding_to_username
    );

    let alice_canister_id_corresponding_to_principal_id = pocket_ic
        .query_call(
            *user_index_canister_id,
            Principal::anonymous(),
            "get_user_canister_id_from_user_principal_id",
            candid::encode_one(alice_principal_id).unwrap(),
        )
        .map(|reply_payload| {
            let alice_principal_id_corresponding_to_username: Option<Principal> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_user_canister_id_from_user_principal_id failed\n"),
                };
            alice_principal_id_corresponding_to_username
        })
        .unwrap();

    assert_eq!(
        alice_canister_id_corresponding_to_username,
        alice_canister_id_corresponding_to_principal_id
    );

    println!(
        "🧪 alice_canister_id_corresponding_to_principal_id: {:?}",
        alice_canister_id_corresponding_to_principal_id
    );
}