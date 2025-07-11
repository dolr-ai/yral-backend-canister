use std::time::Duration;

use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::user_index::types::args::UserIndexInitArgs,
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::{pocket_ic_env::get_new_pocket_ic_env, pocket_ic_init::get_initialized_env_with_provisioned_known_canisters},
    test_constants::{
        get_canister_wasm, get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
    },
};

#[ignore]
#[test]
fn when_canister_cycle_balance_is_below_the_configured_or_freezing_threshold_then_running_topup_function_in_user_index_canister_tops_up_the_canisters_that_need_it() {
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

    let alice_starting_cycle_balance = pocket_ic
        .update_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_user_caniser_cycle_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: u128 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_caniser_cycle_balance failed\n"),
            };
            response
        })
        .unwrap();

    println!(
        "🧪 alice_starting_cycle_balance: {}",
        alice_starting_cycle_balance
    );

    pocket_ic
        .update_call(
            alice_canister_id,
            *user_index_canister_id,
            "return_cycles_to_user_index_canister",
            candid::encode_one(Some(600_000_000_000_u128)).unwrap(),
        )
        .unwrap();

    let alice_reduced_cycle_balance = pocket_ic
        .update_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_user_caniser_cycle_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: u128 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_caniser_cycle_balance failed\n"),
            };
            response
        })
        .unwrap();

    println!(
        "🧪 alice_reduced_cycle_balance: {}",
        alice_reduced_cycle_balance
    );

    assert!(alice_starting_cycle_balance > alice_reduced_cycle_balance);

    pocket_ic
        .upgrade_canister(
            *user_index_canister_id,
            get_canister_wasm(KnownPrincipalType::CanisterIdUserIndex),
            candid::encode_one(UserIndexInitArgs {
                ..Default::default()
            })
            .unwrap(),
            Some(get_global_super_admin_principal_id()),
        )
        .unwrap();

    pocket_ic.advance_time(Duration::from_secs(30));
    pocket_ic.tick();

    let alice_cycle_balance_after_user_index_upgrade = pocket_ic
        .update_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_user_caniser_cycle_balance",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: u128 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_user_caniser_cycle_balance failed\n"),
            };
            response
        })
        .unwrap();

    println!(
        "🧪 alice_cycle_balance_after_user_index_upgrade: {}",
        alice_cycle_balance_after_user_index_upgrade
    );

    assert!(alice_cycle_balance_after_user_index_upgrade >= 600_000_000_000);
}