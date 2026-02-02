use candid::Principal;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontendV7;
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
    get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
};

#[test]
fn test_get_users_profile_details_multiple_users() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;

    let alice = get_mock_user_alice_principal_id();
    let bob = get_mock_user_bob_principal_id();
    let charlie = get_mock_user_charlie_principal_id();
    let dan = get_mock_user_dan_principal_id();

    // Register all users
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
        assert!(
            result.is_ok(),
            "User registration failed for {:?}: {:?}",
            user,
            result
        );
    }

    // Alice follows Bob and Charlie
    update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "follow_user",
        (bob,),
    )
    .expect("Failed to follow user")
    .expect("Follow operation failed");

    update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "follow_user",
        (charlie,),
    )
    .expect("Failed to follow user")
    .expect("Follow operation failed");

    // Bob follows Alice
    update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        bob,
        "follow_user",
        (alice,),
    )
    .expect("Failed to follow user")
    .expect("Follow operation failed");

    // Test get_users_profile_details with multiple users
    let users_to_query = vec![alice, bob, charlie];
    let profiles_result = query::<_, Result<Vec<UserProfileDetailsForFrontendV7>, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_users_profile_details",
        (users_to_query.clone(),),
    )
    .expect("Failed to call get_users_profile_details")
    .expect("Query returned error");

    // Verify the response
    assert_eq!(
        profiles_result.len(),
        3,
        "Should return profiles for all 3 users"
    );

    // Verify Alice's profile
    let alice_profile = profiles_result
        .iter()
        .find(|p| p.principal_id == alice)
        .expect("Alice's profile should be in results");
    assert_eq!(alice_profile.principal_id, alice);
    assert_eq!(alice_profile.followers_count, 1); // Bob follows Alice
    assert_eq!(alice_profile.following_count, 2); // Alice follows Bob and Charlie
    assert_eq!(
        alice_profile.caller_follows_user, None,
        "Caller (Alice) doesn't follow herself"
    );
    assert_eq!(
        alice_profile.user_follows_caller, None,
        "Alice doesn't follow herself"
    );

    // Verify Bob's profile
    let bob_profile = profiles_result
        .iter()
        .find(|p| p.principal_id == bob)
        .expect("Bob's profile should be in results");
    assert_eq!(bob_profile.principal_id, bob);
    assert_eq!(bob_profile.followers_count, 1); // Alice follows Bob
    assert_eq!(bob_profile.following_count, 1); // Bob follows Alice
    assert_eq!(
        bob_profile.caller_follows_user,
        Some(true),
        "Caller (Alice) follows Bob"
    );
    assert_eq!(
        bob_profile.user_follows_caller,
        Some(true),
        "Bob follows caller (Alice)"
    );

    // Verify Charlie's profile
    let charlie_profile = profiles_result
        .iter()
        .find(|p| p.principal_id == charlie)
        .expect("Charlie's profile should be in results");
    assert_eq!(charlie_profile.principal_id, charlie);
    assert_eq!(charlie_profile.followers_count, 1); // Alice follows Charlie
    assert_eq!(charlie_profile.following_count, 0); // Charlie doesn't follow anyone
    assert_eq!(
        charlie_profile.caller_follows_user,
        Some(true),
        "Caller (Alice) follows Charlie"
    );
    assert_eq!(
        charlie_profile.user_follows_caller,
        Some(false),
        "Charlie doesn't follow caller (Alice)"
    );
}

#[test]
fn test_get_users_profile_details_empty_list() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;

    let alice = get_mock_user_alice_principal_id();

    // Register Alice
    update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "register_new_user",
        (),
    )
    .expect("Failed to register user")
    .expect("User registration failed");

    // Test with empty list
    let empty_users: Vec<Principal> = vec![];
    let profiles_result = query::<_, Result<Vec<UserProfileDetailsForFrontendV7>, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_users_profile_details",
        (empty_users,),
    )
    .expect("Failed to call get_users_profile_details")
    .expect("Query returned error");

    assert_eq!(profiles_result.len(), 0, "Should return empty list");
}

#[test]
fn test_get_users_profile_details_single_user() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;

    let alice = get_mock_user_alice_principal_id();
    let bob = get_mock_user_bob_principal_id();

    // Register Alice and Bob
    for user in &[alice, bob] {
        update::<_, Result<(), String>>(
            &pocket_ic,
            user_service_canister,
            *user,
            "register_new_user",
            (),
        )
        .expect("Failed to register user")
        .expect("User registration failed");
    }

    // Test with single user
    let single_user = vec![bob];
    let profiles_result = query::<_, Result<Vec<UserProfileDetailsForFrontendV7>, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_users_profile_details",
        (single_user,),
    )
    .expect("Failed to call get_users_profile_details")
    .expect("Query returned error");

    assert_eq!(profiles_result.len(), 1, "Should return profile for 1 user");
    assert_eq!(
        profiles_result[0].principal_id, bob,
        "Should return Bob's profile"
    );
}

#[test]
fn test_get_users_profile_details_with_unregistered_users() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;

    let alice = get_mock_user_alice_principal_id();
    let bob = get_mock_user_bob_principal_id();
    let unregistered_user = get_mock_user_charlie_principal_id();

    // Register only Alice and Bob
    for user in &[alice, bob] {
        update::<_, Result<(), String>>(
            &pocket_ic,
            user_service_canister,
            *user,
            "register_new_user",
            (),
        )
        .expect("Failed to register user")
        .expect("User registration failed");
    }

    // Test with mix of registered and unregistered users
    let users_to_query = vec![alice, bob, unregistered_user];
    let profiles = query::<_, Result<Vec<UserProfileDetailsForFrontendV7>, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_users_profile_details",
        (users_to_query,),
    )
    .expect("query call failed")
    .expect("Query returned error");

    assert!(profiles.len() <= 2, "Should only return registered users");
    assert!(
        profiles
            .iter()
            .all(|p| p.principal_id == alice || p.principal_id == bob),
        "Should only contain Alice and Bob"
    );
}

#[test]
fn test_get_users_profile_details_preserves_input_order() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;

    let alice = get_mock_user_alice_principal_id();
    let bob = get_mock_user_bob_principal_id();
    let charlie = get_mock_user_charlie_principal_id();
    let dan = get_mock_user_dan_principal_id();

    // Register all users
    let users = vec![alice, bob, charlie, dan];
    for user in &users {
        update::<_, Result<(), String>>(
            &pocket_ic,
            user_service_canister,
            *user,
            "register_new_user",
            (),
        )
        .expect("Failed to register user")
        .expect("User registration failed");
    }

    // Test with different orderings to ensure output matches input order

    // Test order 1: alice, bob, charlie, dan
    let order1 = vec![alice, bob, charlie, dan];
    let profiles1 = query::<_, Result<Vec<UserProfileDetailsForFrontendV7>, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_users_profile_details",
        (order1.clone(),),
    )
    .expect("Failed to call get_users_profile_details")
    .expect("Query returned error");

    assert_eq!(profiles1.len(), 4, "Should return 4 profiles");
    for (i, expected_principal) in order1.iter().enumerate() {
        assert_eq!(
            profiles1[i].principal_id, *expected_principal,
            "Profile at index {} should match principal at index {}",
            i, i
        );
    }

    // Test order 2: dan, charlie, bob, alice (reversed)
    let order2 = vec![dan, charlie, bob, alice];
    let profiles2 = query::<_, Result<Vec<UserProfileDetailsForFrontendV7>, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_users_profile_details",
        (order2.clone(),),
    )
    .expect("Failed to call get_users_profile_details")
    .expect("Query returned error");

    assert_eq!(profiles2.len(), 4, "Should return 4 profiles");
    for (i, expected_principal) in order2.iter().enumerate() {
        assert_eq!(
            profiles2[i].principal_id, *expected_principal,
            "Profile at index {} should match principal at index {} in reversed order",
            i, i
        );
    }

    // Test order 3: charlie, alice, dan, bob (random order)
    let order3 = vec![charlie, alice, dan, bob];
    let profiles3 = query::<_, Result<Vec<UserProfileDetailsForFrontendV7>, String>>(
        &pocket_ic,
        user_service_canister,
        alice,
        "get_users_profile_details",
        (order3.clone(),),
    )
    .expect("Failed to call get_users_profile_details")
    .expect("Query returned error");

    assert_eq!(profiles3.len(), 4, "Should return 4 profiles");
    for (i, expected_principal) in order3.iter().enumerate() {
        assert_eq!(
            profiles3[i].principal_id, *expected_principal,
            "Profile at index {} should match principal at index {} in random order",
            i, i
        );
    }
}
