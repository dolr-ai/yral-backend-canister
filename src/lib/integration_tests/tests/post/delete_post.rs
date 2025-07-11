
use candid::{encode_one, Principal};
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        error::GetPostsOfUserProfileError,
        post::{PostDetailsForFrontend, PostDetailsFromFrontend},
    },
    common::types::known_principal::KnownPrincipalType,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    },
};

#[test]
fn test_user_can_delete_its_own_post() {
    let (pic, known_principals) = get_new_pocket_ic_env();

    let alice_principal = get_mock_user_alice_principal_id();
    let bob_principal = get_mock_user_bob_principal_id();

    let platform_canister_id = known_principals
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let global_admin = known_principals
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pic.topology().get_app_subnets();

    let subnet_orchestrator_canister_id = pic
        .update_call(
            platform_canister_id,
            global_admin,
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
        pic.tick()
    }

    let alice_canister_id = pic
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

    let post_details = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let post_id = pic
        .update_call(
            alice_canister_id,
            alice_principal,
            "add_post_v2",
            encode_one(post_details).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    let posts = pic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_posts_of_this_user_profile_with_pagination_cursor",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile failed\n"),
                };
            profile.unwrap()
        })
        .unwrap();

    assert_eq!(posts.len(), 1);

    let post_details_for_frontend = pic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_individual_post_details_by_id",
            candid::encode_args((post_id,)).unwrap(),
        )
        .map(|reply_payload| {
            let post_details: PostDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_individual_post_details_by_id failed\n"),
            };
            post_details
        })
        .unwrap();

    assert_eq!(post_details_for_frontend.id, post_id);

    /// Only profile owner can delete canister.
    let delete_result = pic
        .update_call(
            alice_canister_id,
            bob_principal,
            "delete_post",
            candid::encode_one(post_id).unwrap(),
        )
        .unwrap();

    assert!(matches!(delete_result, WasmResult::Reject(_)));

    pic.update_call(
        alice_canister_id,
        alice_principal,
        "delete_post",
        candid::encode_one(post_id).unwrap(),
    )
    .map(|reply_payload| {
        let result: Result<(), String> = match reply_payload {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 delete_post failed\n"),
        };
        result
    })
    .unwrap()
    .unwrap();

    let posts = pic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_posts_of_this_user_profile_with_pagination_cursor",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile failed\n"),
                };
            profile.unwrap()
        })
        .unwrap();

    assert_eq!(posts.len(), 0);

    let post_details_for_frontend = pic
        .query_call(
            alice_canister_id,
            Principal::anonymous(),
            "get_individual_post_details_by_id",
            candid::encode_args((post_id,)).unwrap(),
        )
        .map(|reply_payload| {
            let post_details: PostDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_individual_post_details_by_id failed\n"),
            };
            post_details
        });

    assert!(post_details_for_frontend.is_err())
}
