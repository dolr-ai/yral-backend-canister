use candid::Encode;
use pocket_ic::WasmResult;
use shared_utils::{
    canister_specific::user_post_service::types::{
        error::UserPostServiceError,
        storage::Post,
    },
    common::{types::top_posts::post_score_index_item::PostStatus, utils::system_time::get_current_system_time_from_ic},
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned,
    test_constants::{get_mock_user_alice_principal_id, get_mock_user_bob_principal_id},
};
use std::collections::HashSet;

#[test]
fn test_add_and_get_post() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;

    let alice_principal = get_mock_user_alice_principal_id();
    
    // Create a test post
    let test_post = Post {
        id: 999,
        creator_principal: alice_principal,
        video_uid: "test_video_123".to_string(),
        description: "Test post description".to_string(),
        hashtags: vec!["test".to_string(), "integration".to_string()],
        status: PostStatus::ReadyToView,
        created_at: get_current_system_time_from_ic(),
        likes: HashSet::new(),
        share_count: 0,
        view_stats: Default::default(),
        is_nsfw: false,
    };

    // Add post
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
    assert_eq!(post_id, 0); // First post should have ID 0

    // Get the post back
    let res = pic.query_call(
        user_post_service_canister_id,
        alice_principal,
        "get_post",
        candid::encode_one(post_id).unwrap()
    ).unwrap();
    let retrieved_post: Option<Post> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get post failed\n"),
    };
    
    let retrieved_post = retrieved_post.expect("Post should exist");
    assert_eq!(retrieved_post.id, 999); // Video ID from CF Streams should remain unchanged
    assert_eq!(retrieved_post.creator_principal, alice_principal);
    assert_eq!(retrieved_post.video_uid, "test_video_123");
    assert_eq!(retrieved_post.description, "Test post description");
    assert_eq!(retrieved_post.hashtags, vec!["test".to_string(), "integration".to_string()]);
}

#[test]
fn test_get_posts_by_creator() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    
    // Add multiple posts
    for i in 0..5 {
        let test_post = Post {
            id: 999,
            creator_principal: alice_principal,
            video_uid: format!("video_{}", i),
            description: format!("Post {}", i),
            hashtags: vec![],
            status: PostStatus::ReadyToView,
            created_at: get_current_system_time_from_ic(),
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
        post_id.unwrap();
    }

    // Get posts by creator with pagination
    let res = pic.query_call(
        user_post_service_canister_id,
        alice_principal,
        "get_posts_by_creator",
        Encode!(&alice_principal, &3usize, &0usize).unwrap() // limit: 3, offset: 0
    ).unwrap();
    let posts: Vec<Post> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get posts by creator failed\n"),
    };
    
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].video_uid, "video_0");
    assert_eq!(posts[1].video_uid, "video_1");
    assert_eq!(posts[2].video_uid, "video_2");

    // Test offset
    let res = pic.query_call(
        user_post_service_canister_id,
        alice_principal,
        "get_posts_by_creator",
        Encode!(&alice_principal, &3usize, &2usize).unwrap() // limit: 3, offset: 2
    ).unwrap();
    let posts: Vec<Post> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get posts by creator failed\n"),
    };
    
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].video_uid, "video_2");
    assert_eq!(posts[1].video_uid, "video_3");
    assert_eq!(posts[2].video_uid, "video_4");
}

#[test]
fn test_post_id_increment() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    let bob_principal = get_mock_user_bob_principal_id();
    
    // Add posts from different users
    let test_post = Post {
        id: 999,
        creator_principal: alice_principal,
        video_uid: "alice_video".to_string(),
        description: "Alice's post".to_string(),
        hashtags: vec![],
        status: PostStatus::ReadyToView,
        created_at: get_current_system_time_from_ic(),
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
    let alice_post_id: Result<u64, UserPostServiceError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add post failed\n"),
    };
    assert_eq!(alice_post_id.unwrap(), 0);

    // Bob's post
    let test_post = Post {
        id: 999,
        creator_principal: bob_principal,
        video_uid: "bob_video".to_string(),
        description: "Bob's post".to_string(),
        hashtags: vec![],
        status: PostStatus::ReadyToView,
        created_at: get_current_system_time_from_ic(),
        likes: HashSet::new(),
        share_count: 0,
        view_stats: Default::default(),
        is_nsfw: false,
    };

    let res = pic.update_call(
        user_post_service_canister_id,
        bob_principal,
        "add_post",
        Encode!(&bob_principal, &test_post).unwrap()
    ).unwrap();
    let bob_post_id: Result<u64, UserPostServiceError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add post failed\n"),
    };
    assert_eq!(bob_post_id.unwrap(), 1);
}

#[test]
fn test_nonexistent_post() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    
    // Try to get a non-existent post
    let res = pic.query_call(
        user_post_service_canister_id,
        alice_principal,
        "get_post",
        candid::encode_one(999u64).unwrap()
    ).unwrap();
    let post: Option<Post> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get post failed\n"),
    };
    
    assert!(post.is_none());
}