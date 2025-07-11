use candid::Principal;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::post::{
        PostDetailsForFrontend, PostDetailsFromFrontend,
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::{pocket_ic_env::get_new_pocket_ic_env, pocket_ic_init::get_initialized_env_with_provisioned_known_canisters},
    test_constants::get_mock_user_alice_principal_id,
};

#[ignore]
#[test]
fn when_creating_a_new_post_then_post_score_should_be_calculated() {
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

    let newly_created_post_id = pocket_ic
        .update_call(
            alice_canister_id,
            alice_principal_id,
            "add_post_v2",
            candid::encode_args((PostDetailsFromFrontend {
                is_nsfw: false,
                description: "This is a fun video to watch".to_string(),
                hashtags: vec!["fun".to_string(), "video".to_string()],
                video_uid: "abcd#1234".to_string(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },))
            .unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    let post_score = pocket_ic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_individual_post_details_by_id",
            candid::encode_args((newly_created_post_id,)).unwrap(),
        )
        .map(|reply_payload| {
            let post_details: PostDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_individual_post_details_by_id failed\n"),
            };
            post_details.home_feed_ranking_score
        })
        .unwrap();

    assert!(post_score > 0);
}