use candid::Principal;
use pocket_ic::PocketIc;
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
};
use user_info_service::api::get_followers::FollowersResponse;
use user_info_service::api::get_following::FollowingResponse;

#[test]
fn test_comprehensive_follower_following_functionality() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;

    let alice = get_mock_user_alice_principal_id();
    let bob = get_mock_user_bob_principal_id();
    let charlie = get_mock_user_charlie_principal_id();
    let dan = get_mock_user_dan_principal_id();

    // Setup: Register all users
    let users = vec![alice, bob, charlie, dan];
    for user in &users {
        let result = update::<_, Result<(), String>>(
            &pocket_ic,
            user_service_canister,
            *user,
            "register_new_user",
            (),
        )
        .expect("Failed to register user");
        assert!(result.is_ok(), "User registration failed for {:?}: {:?}", user, result);
    }

    // Test Follow Operations
    
    // Alice follows Bob
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "follow_user",
        (bob,),
    )
    .expect("Failed to call follow_user");
    assert!(result.is_ok(), "Alice should be able to follow Bob: {:?}", result);

    // Alice follows Charlie
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "follow_user",
        (charlie,),
    )
    .expect("Failed to call follow_user");
    assert!(result.is_ok(), "Alice should be able to follow Charlie: {:?}", result);

    // Bob follows Alice
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        bob,
        "follow_user",
        (alice,),
    )
    .expect("Failed to call follow_user");
    assert!(result.is_ok(), "Bob should be able to follow Alice: {:?}", result);

    // Charlie follows Alice
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        charlie,
        "follow_user",
        (alice,),
    )
    .expect("Failed to call follow_user");
    assert!(result.is_ok(), "Charlie should be able to follow Alice: {:?}", result);

    // Charlie follows Dan
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        charlie,
        "follow_user",
        (dan,),
    )
    .expect("Failed to call follow_user");
    assert!(result.is_ok(), "Charlie should be able to follow Dan: {:?}", result);

    // Test validation: Cannot follow yourself
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "follow_user",
        (alice,),
    )
    .expect("Failed to call follow_user");
    assert!(result.is_err(), "Should not be able to follow yourself");
    assert_eq!(result.unwrap_err(), "Cannot follow yourself");

    // Test validation: Cannot follow same user twice
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "follow_user",
        (bob,),
    )
    .expect("Failed to call follow_user");
    assert!(result.is_err(), "Should not be able to follow same user twice");
    assert_eq!(result.unwrap_err(), "Already following this user");

    // Test validation: Cannot follow non-existent user
    let non_existent = Principal::from_text("2vxsx-fae").unwrap();
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "follow_user",
        (non_existent,),
    )
    .expect("Failed to call follow_user");
    assert!(result.is_err(), "Should not be able to follow non-existent user");
    assert_eq!(result.unwrap_err(), "Target user not found");

    // Verify Followers
    
    // Alice should have 2 followers: Bob and Charlie
    let alice_followers = query::<_, Result<FollowersResponse, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_followers",
        (alice, 0u64, 10u64),
    )
    .expect("Failed to query followers")
    .expect("Failed to get Alice's followers");
    
    assert_eq!(alice_followers.total_count, 2, "Alice should have 2 followers");
    assert!(alice_followers.followers.contains(&bob), "Bob should be in Alice's followers");
    assert!(alice_followers.followers.contains(&charlie), "Charlie should be in Alice's followers");

    // Bob should have 1 follower: Alice
    let bob_followers = query::<_, Result<FollowersResponse, String>>(
        &pocket_ic,
        user_service_canister,
        bob,
        "get_followers",
        (bob, 0u64, 10u64),
    )
    .expect("Failed to query followers")
    .expect("Failed to get Bob's followers");
    
    assert_eq!(bob_followers.total_count, 1, "Bob should have 1 follower");
    assert!(bob_followers.followers.contains(&alice), "Alice should be in Bob's followers");

    // Charlie should have 1 follower: Alice
    let charlie_followers = query::<_, Result<FollowersResponse, String>>(
        &pocket_ic,
        user_service_canister,
        charlie,
        "get_followers",
        (charlie, 0u64, 10u64),
    )
    .expect("Failed to query followers")
    .expect("Failed to get Charlie's followers");
    
    assert_eq!(charlie_followers.total_count, 1, "Charlie should have 1 follower");
    assert!(charlie_followers.followers.contains(&alice), "Alice should be in Charlie's followers");

    // Dan should have 1 follower: Charlie
    let dan_followers = query::<_, Result<FollowersResponse, String>>(
        &pocket_ic,
        user_service_canister,
        dan,
        "get_followers",
        (dan, 0u64, 10u64),
    )
    .expect("Failed to query followers")
    .expect("Failed to get Dan's followers");
    
    assert_eq!(dan_followers.total_count, 1, "Dan should have 1 follower");
    assert!(dan_followers.followers.contains(&charlie), "Charlie should be in Dan's followers");

    // Verify Following
    
    // Alice should be following 2: Bob and Charlie
    let alice_following = query::<_, Result<FollowingResponse, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_following",
        (alice, 0u64, 10u64),
    )
    .expect("Failed to query following")
    .expect("Failed to get Alice's following");
    
    assert_eq!(alice_following.total_count, 2, "Alice should be following 2 users");
    assert!(alice_following.following.contains(&bob), "Alice should be following Bob");
    assert!(alice_following.following.contains(&charlie), "Alice should be following Charlie");

    // Bob should be following 1: Alice
    let bob_following = query::<_, Result<FollowingResponse, String>>(
        &pocket_ic,
        user_service_canister,
        bob,
        "get_following",
        (bob, 0u64, 10u64),
    )
    .expect("Failed to query following")
    .expect("Failed to get Bob's following");
    
    assert_eq!(bob_following.total_count, 1, "Bob should be following 1 user");
    assert!(bob_following.following.contains(&alice), "Bob should be following Alice");

    // Charlie should be following 2: Alice and Dan
    let charlie_following = query::<_, Result<FollowingResponse, String>>(
        &pocket_ic,
        user_service_canister,
        charlie,
        "get_following",
        (charlie, 0u64, 10u64),
    )
    .expect("Failed to query following")
    .expect("Failed to get Charlie's following");
    
    assert_eq!(charlie_following.total_count, 2, "Charlie should be following 2 users");
    assert!(charlie_following.following.contains(&alice), "Charlie should be following Alice");
    assert!(charlie_following.following.contains(&dan), "Charlie should be following Dan");

    // Dan should be following 0
    let dan_following = query::<_, Result<FollowingResponse, String>>(
        &pocket_ic,
        user_service_canister,
        dan,
        "get_following",
        (dan, 0u64, 10u64),
    )
    .expect("Failed to query following")
    .expect("Failed to get Dan's following");
    
    assert_eq!(dan_following.total_count, 0, "Dan should not be following anyone");
    assert!(dan_following.following.is_empty(), "Dan's following list should be empty");

    // Test Pagination
    
    // Test pagination with offset
    let alice_followers_page2 = query::<_, Result<FollowersResponse, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_followers",
        (alice, 1u64, 1u64),
    )
    .expect("Failed to query followers")
    .expect("Failed to get Alice's followers with pagination");
    
    assert_eq!(alice_followers_page2.followers.len(), 1, "Should get 1 follower with limit 1");
    assert_eq!(alice_followers_page2.total_count, 2, "Total count should still be 2");

    // Test Unfollow Operations
    
    // Alice unfollows Bob
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "unfollow_user",
        (bob,),
    )
    .expect("Failed to call unfollow_user");
    assert!(result.is_ok(), "Alice should be able to unfollow Bob: {:?}", result);

    // Charlie unfollows Alice
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        charlie,
        "unfollow_user",
        (alice,),
    )
    .expect("Failed to call unfollow_user");
    assert!(result.is_ok(), "Charlie should be able to unfollow Alice: {:?}", result);

    // Test validation: Cannot unfollow someone you're not following
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "unfollow_user",
        (dan,),
    )
    .expect("Failed to call unfollow_user");
    assert!(result.is_err(), "Should not be able to unfollow someone you're not following");
    assert_eq!(result.unwrap_err(), "Not following this user");

    // Test validation: Cannot unfollow yourself
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "unfollow_user",
        (alice,),
    )
    .expect("Failed to call unfollow_user");
    assert!(result.is_err(), "Should not be able to unfollow yourself");
    assert_eq!(result.unwrap_err(), "Cannot unfollow yourself");

    // Final Verification after unfollows
    
    // Alice should now have 1 follower: Bob (Charlie unfollowed)
    let alice_followers_final = query::<_, Result<FollowersResponse, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_followers",
        (alice, 0u64, 10u64),
    )
    .expect("Failed to query followers")
    .expect("Failed to get Alice's final followers");
    
    assert_eq!(alice_followers_final.total_count, 1, "Alice should have 1 follower after unfollows");
    assert!(alice_followers_final.followers.contains(&bob), "Bob should still be in Alice's followers");
    assert!(!alice_followers_final.followers.contains(&charlie), "Charlie should not be in Alice's followers");

    // Alice should now be following 1: Charlie (unfollowed Bob)
    let alice_following_final = query::<_, Result<FollowingResponse, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_following",
        (alice, 0u64, 10u64),
    )
    .expect("Failed to query following")
    .expect("Failed to get Alice's final following");
    
    assert_eq!(alice_following_final.total_count, 1, "Alice should be following 1 user after unfollows");
    assert!(!alice_following_final.following.contains(&bob), "Alice should not be following Bob");
    assert!(alice_following_final.following.contains(&charlie), "Alice should still be following Charlie");

    // Bob should have 0 followers (Alice unfollowed)
    let bob_followers_final = query::<_, Result<FollowersResponse, String>>(
        &pocket_ic,
        user_service_canister,
        bob,
        "get_followers",
        (bob, 0u64, 10u64),
    )
    .expect("Failed to query followers")
    .expect("Failed to get Bob's final followers");
    
    assert_eq!(bob_followers_final.total_count, 0, "Bob should have 0 followers after Alice unfollowed");
    assert!(bob_followers_final.followers.is_empty(), "Bob's followers list should be empty");

    // Charlie should be following 1: Dan (unfollowed Alice)
    let charlie_following_final = query::<_, Result<FollowingResponse, String>>(
        &pocket_ic,
        user_service_canister,
        charlie,
        "get_following",
        (charlie, 0u64, 10u64),
    )
    .expect("Failed to query following")
    .expect("Failed to get Charlie's final following");
    
    assert_eq!(charlie_following_final.total_count, 1, "Charlie should be following 1 user after unfollows");
    assert!(!charlie_following_final.following.contains(&alice), "Charlie should not be following Alice");
    assert!(charlie_following_final.following.contains(&dan), "Charlie should still be following Dan");

    println!("✅ All follower/following functionality tests passed!");
}