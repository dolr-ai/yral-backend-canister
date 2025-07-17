use candid::Encode;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::user_post_service::types::{
        error::UserPostServiceError,
        storage::Post,
    },
    common::types::top_posts::post_score_index_item::PostStatus,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned,
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_charlie_principal_id},
};
use std::time::SystemTime;
use std::collections::HashSet;

#[test]
fn test_add_post_authorization() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    let charlie_principal = get_mock_user_charlie_principal_id();
    
    // Create a test post for Alice
    let test_post = Post {
        id: 0,
        creator_principal: alice_principal,
        video_uid: "alice_video".to_string(),
        description: "Alice's post".to_string(),
        hashtags: vec![],
        status: PostStatus::ReadyToView,
        created_at: SystemTime::now(),
        likes: HashSet::new(),
        share_count: 0,
        view_stats: Default::default(),
        is_nsfw: false,
    };

    // Alice can add her own post
    let res = pic.update_call(
        user_post_service_canister_id,
        alice_principal,
        "add_post",
        Encode!(&alice_principal, &test_post).unwrap()
    ).unwrap();
    let post_id: Result<u64, UserPostServiceError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add post failed\n"),
    };
    assert!(post_id.is_ok());

    // Charlie cannot add a post for Alice
    let res = pic.update_call(
        user_post_service_canister_id,
        charlie_principal,
        "add_post",
        Encode!(&alice_principal, &test_post).unwrap()
    ).unwrap();
    let result: Result<u64, UserPostServiceError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add post failed\n"),
    };
    
    // Should return Unauthorized error
    assert!(matches!(result, Err(UserPostServiceError::Unauthorized)));
}

#[test]
fn test_controller_can_add_post() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let controller_principal = pic.get_controllers(&user_post_service_canister_id)[0];
    let alice_principal = get_mock_user_alice_principal_id();
    
    // Create a test post for Alice
    let test_post = Post {
        id: 0,
        creator_principal: alice_principal,
        video_uid: "controller_added_video".to_string(),
        description: "Post added by controller".to_string(),
        hashtags: vec![],
        status: PostStatus::ReadyToView,
        created_at: SystemTime::now(),
        likes: HashSet::new(),
        share_count: 0,
        view_stats: Default::default(),
        is_nsfw: false,
    };

    // Controller can add a post for any user
    let res = pic.update_call(
        user_post_service_canister_id,
        controller_principal,
        "add_post",
        Encode!(&alice_principal, &test_post).unwrap()
    ).unwrap();
    let post_id: Result<u64, UserPostServiceError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add post failed\n"),
    };
    assert!(post_id.is_ok());
}

#[test]
fn test_get_posts_is_public() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    let charlie_principal = get_mock_user_charlie_principal_id();
    
    // Alice adds a post
    let test_post = Post {
        id: 0,
        creator_principal: alice_principal,
        video_uid: "alice_public_video".to_string(),
        description: "Alice's public post".to_string(),
        hashtags: vec![],
        status: PostStatus::ReadyToView,
        created_at: SystemTime::now(),
        likes: HashSet::new(),
        share_count: 0,
        view_stats: Default::default(),
        is_nsfw: false,
    };

    let res = pic.update_call(
        user_post_service_canister_id,
        alice_principal,
        "add_post",
        Encode!(&alice_principal, &test_post).unwrap()
    ).unwrap();
    let post_id: Result<u64, UserPostServiceError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add post failed\n"),
    };
    let post_id = post_id.unwrap();

    // Charlie can get Alice's post by ID (public query)
    let res = pic.query_call(
        user_post_service_canister_id,
        charlie_principal,
        "get_post",
        candid::encode_one(post_id).unwrap()
    ).unwrap();
    let retrieved_post: Option<Post> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get post failed\n"),
    };
    
    assert!(retrieved_post.is_some());
    assert_eq!(retrieved_post.unwrap().video_uid, "alice_public_video");

    // Charlie can also get posts by creator (public query)
    let res = pic.query_call(
        user_post_service_canister_id,
        charlie_principal,
        "get_posts_by_creator",
        Encode!(&alice_principal, &10usize, &0usize).unwrap()
    ).unwrap();
    let posts: Vec<Post> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get posts by creator failed\n"),
    };
    
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].video_uid, "alice_public_video");
}