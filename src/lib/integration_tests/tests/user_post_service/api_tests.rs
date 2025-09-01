use shared_utils::canister_specific::user_post_service::types::{
    args::{PostDetailsForFrontend, PostDetailsFromFrontend},
    error::UserPostServiceError,
    storage::{Post, PostViewDetailsFromFrontend},
};
use test_utils::{
    canister_calls::{query, update},
    setup::{
        env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned,
        test_constants::{
            get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
            get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id,
        },
    },
};

// Helper function to create a test post
fn create_test_post_details(
    id: &str,
    creator_principal: candid::Principal,
    video_uid: &str,
    description: &str,
) -> PostDetailsFromFrontend {
    PostDetailsFromFrontend {
        id: id.to_string(),
        creator_principal,
        video_uid: video_uid.to_string(),
        description: description.to_string(),
        hashtags: vec!["test".to_string(), "integration".to_string()],
    }
}

// Helper function to add a post as admin
fn add_post_as_admin(
    pic: &pocket_ic::PocketIc,
    user_post_service_canister_id: candid::Principal,
    post_details: PostDetailsFromFrontend,
) -> Result<(), UserPostServiceError> {
    let admin_principal = get_global_super_admin_principal_id();
    let result: Result<Result<(), UserPostServiceError>, Box<dyn std::error::Error>> = update(
        pic,
        user_post_service_canister_id,
        admin_principal,
        "add_post",
        (post_details,),
    );

    match result {
        Ok(inner_result) => inner_result,
        Err(e) => {
            eprintln!("Error calling add_post: {}", e);
            Err(UserPostServiceError::Unauthorized)
        }
    }
}

#[test]
fn test_add_post_success() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    let post_details = create_test_post_details(
        "test_post_1",
        alice_principal,
        "video_123",
        "Test post description",
    );

    let result = add_post_as_admin(&pic, user_post_service_canister_id, post_details);
    assert!(result.is_ok());
}

#[test]
fn test_add_post_unauthorized() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    let post_details = create_test_post_details(
        "test_post_1",
        alice_principal,
        "video_123",
        "Test post description",
    );

    // Try to add post as regular user (should fail)
    let result: Result<Result<(), UserPostServiceError>, Box<dyn std::error::Error>> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "add_post",
        (post_details,),
    );

    // Should fail at the transport level due to unauthorized access
    assert!(result.is_err());
}

#[test]
fn test_add_duplicate_post() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    let post_details = create_test_post_details(
        "duplicate_post",
        alice_principal,
        "video_123",
        "Test post description",
    );

    // Add post first time - should succeed
    let result = add_post_as_admin(&pic, user_post_service_canister_id, post_details.clone());
    assert!(result.is_ok());

    // Try to add same post again - should fail
    let result = add_post_as_admin(&pic, user_post_service_canister_id, post_details);
    assert!(result.is_err());
}

#[test]
fn test_get_individual_post_details_by_id_success() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "get_test_post",
        alice_principal,
        "video_456",
        "Post to retrieve",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Get the post
    let retrieved_post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("get_test_post",),
    )
    .unwrap();

    let post = retrieved_post.unwrap();
    assert_eq!(post.id, "get_test_post");
    assert_eq!(post.creator_principal, alice_principal);
    assert_eq!(post.video_uid, "video_456");
    assert_eq!(post.description, "Post to retrieve");
    assert_eq!(
        post.hashtags,
        vec!["test".to_string(), "integration".to_string()]
    );
    assert_eq!(post.share_count, 0);
    assert!(post.likes.is_empty());
}

#[test]
fn test_get_individual_post_details_by_id_not_found() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    // Try to get a non-existent post
    let result: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("non_existent_post",),
    )
    .unwrap();

    assert!(matches!(result, Err(UserPostServiceError::PostNotFound)));
}

#[test]
fn test_get_posts_of_this_user_profile_with_pagination_cursor() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    let bob_principal = get_mock_user_bob_principal_id();

    // Add multiple posts for Alice
    for i in 0..5 {
        let post_details = create_test_post_details(
            &format!("alice_post_{}", i),
            alice_principal,
            &format!("video_alice_{}", i),
            &format!("Alice's post #{}", i),
        );
        let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);
    }

    // Add multiple posts for Bob
    for i in 0..3 {
        let post_details = create_test_post_details(
            &format!("bob_post_{}", i),
            bob_principal,
            &format!("video_bob_{}", i),
            &format!("Bob's post #{}", i),
        );
        let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);
    }

    // Get Alice's posts with pagination
    let posts: Vec<Post> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_posts_of_this_user_profile_with_pagination_cursor",
        (alice_principal, 3usize, 0usize), // limit: 3, offset: 0
    )
    .unwrap();

    assert_eq!(posts.len(), 3);
    // Verify they are Alice's posts
    for post in &posts {
        assert_eq!(post.creator_principal, alice_principal);
        assert!(post.id.starts_with("alice_post_"));
    }

    // Test offset
    let posts: Vec<Post> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_posts_of_this_user_profile_with_pagination_cursor",
        (alice_principal, 3usize, 2usize), // limit: 3, offset: 2
    )
    .unwrap();

    assert_eq!(posts.len(), 3);
    for post in &posts {
        assert_eq!(post.creator_principal, alice_principal);
    }

    // Get Bob's posts
    let posts: Vec<Post> = query(
        &pic,
        user_post_service_canister_id,
        bob_principal,
        "get_posts_of_this_user_profile_with_pagination_cursor",
        (bob_principal, 10usize, 0usize),
    )
    .unwrap();

    assert_eq!(posts.len(), 3);
    for post in &posts {
        assert_eq!(post.creator_principal, bob_principal);
        assert!(post.id.starts_with("bob_post_"));
    }
}

#[test]
fn test_update_post_add_view_details_watched_partially() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "view_test_post",
        alice_principal,
        "video_view",
        "Post for view testing",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Add view details
    let view_details = PostViewDetailsFromFrontend::WatchedPartially {
        percentage_watched: 50,
    };

    let result: Result<(), UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "update_post_add_view_details",
        ("view_test_post", view_details),
    )
    .unwrap();

    assert!(result.is_ok());

    // Verify the view stats were updated
    let post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("view_test_post",),
    )
    .unwrap();

    let post = post.unwrap();
    assert_eq!(post.view_stats.total_view_count, 1);
    assert_eq!(post.view_stats.threshold_view_count, 1); // 50% > 20%
    assert_eq!(post.view_stats.average_watch_percentage, 50);
}

#[test]
fn test_update_post_add_view_details_watched_multiple_times() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "multi_view_test_post",
        alice_principal,
        "video_multi_view",
        "Post for multiple view testing",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Add view details for multiple watches
    let view_details = PostViewDetailsFromFrontend::WatchedMultipleTimes {
        watch_count: 3,
        percentage_watched: 80,
    };

    let result: Result<(), UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "update_post_add_view_details",
        ("multi_view_test_post", view_details),
    )
    .unwrap();

    assert!(result.is_ok());

    // Verify the view stats were updated
    let post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("multi_view_test_post",),
    )
    .unwrap();

    let post = post.unwrap();
    assert_eq!(post.view_stats.total_view_count, 4); // 3 + 1
    assert_eq!(post.view_stats.threshold_view_count, 4); // All views > 20%
                                                         // Average should consider 3 full views (100%) + 1 partial view (80%)
    assert_eq!(post.view_stats.average_watch_percentage, 95); // (100*3 + 80)/4 = 95
}

#[test]
fn test_update_post_add_view_details_nonexistent_post() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    let view_details = PostViewDetailsFromFrontend::WatchedPartially {
        percentage_watched: 50,
    };

    let result: Result<(), UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "update_post_add_view_details",
        ("nonexistent_post", view_details),
    )
    .unwrap();

    assert!(matches!(result, Err(UserPostServiceError::PostNotFound)));
}

#[test]
fn test_update_post_increment_share_count() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "share_test_post",
        alice_principal,
        "video_share",
        "Post for share testing",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Increment share count
    let share_count: Result<u64, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "update_post_increment_share_count",
        ("share_test_post",),
    )
    .unwrap();

    assert_eq!(share_count.unwrap(), 1);

    // Increment again
    let share_count: Result<u64, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "update_post_increment_share_count",
        ("share_test_post",),
    )
    .unwrap();

    assert_eq!(share_count.unwrap(), 2);

    // Verify the post has the updated share count
    let post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("share_test_post",),
    )
    .unwrap();

    let post = post.unwrap();
    assert_eq!(post.share_count, 2);
}

#[test]
fn test_update_post_increment_share_count_nonexistent_post() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    let result: Result<u64, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "update_post_increment_share_count",
        ("nonexistent_post",),
    )
    .unwrap();

    assert!(matches!(result, Err(UserPostServiceError::PostNotFound)));
}

#[test]
fn test_update_post_toggle_like_status_by_caller() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    let bob_principal = get_mock_user_bob_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "like_test_post",
        alice_principal,
        "video_like",
        "Post for like testing",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Bob likes the post
    let liked: Result<bool, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        bob_principal,
        "update_post_toggle_like_status_by_caller",
        ("like_test_post",),
    )
    .unwrap();

    assert_eq!(liked.unwrap(), true);

    // Verify the post has the like
    let post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("like_test_post",),
    )
    .unwrap();

    let post = post.unwrap();
    assert_eq!(post.likes.len(), 1);
    assert!(post.likes.contains(&bob_principal));

    // Bob unlikes the post
    let liked: Result<bool, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        bob_principal,
        "update_post_toggle_like_status_by_caller",
        ("like_test_post",),
    )
    .unwrap();

    assert_eq!(liked.unwrap(), false);

    // Verify the like is removed
    let post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("like_test_post",),
    )
    .unwrap();

    let post = post.unwrap();
    assert_eq!(post.likes.len(), 0);
}

#[test]
fn test_update_post_toggle_like_status_multiple_users() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    let bob_principal = get_mock_user_bob_principal_id();
    let charlie_principal = get_mock_user_charlie_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "multi_like_test_post",
        alice_principal,
        "video_multi_like",
        "Post for multiple like testing",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Bob likes the post
    let liked: Result<bool, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        bob_principal,
        "update_post_toggle_like_status_by_caller",
        ("multi_like_test_post",),
    )
    .unwrap();
    assert_eq!(liked.unwrap(), true);

    // Charlie likes the post
    let liked: Result<bool, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        charlie_principal,
        "update_post_toggle_like_status_by_caller",
        ("multi_like_test_post",),
    )
    .unwrap();
    assert_eq!(liked.unwrap(), true);

    // Alice likes her own post
    let liked: Result<bool, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "update_post_toggle_like_status_by_caller",
        ("multi_like_test_post",),
    )
    .unwrap();
    assert_eq!(liked.unwrap(), true);

    // Verify all likes are present
    let post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("multi_like_test_post",),
    )
    .unwrap();

    let post = post.unwrap();
    assert_eq!(post.likes.len(), 3);
    assert!(post.likes.contains(&alice_principal));
    assert!(post.likes.contains(&bob_principal));
    assert!(post.likes.contains(&charlie_principal));
}

#[test]
fn test_update_post_toggle_like_status_nonexistent_post() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    let result: Result<bool, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "update_post_toggle_like_status_by_caller",
        ("nonexistent_post",),
    )
    .unwrap();

    assert!(matches!(result, Err(UserPostServiceError::PostNotFound)));
}

#[test]
fn test_delete_post_success() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "delete_test_post",
        alice_principal,
        "video_delete",
        "Post to delete",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Verify the post exists
    let post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("delete_test_post",),
    )
    .unwrap();
    assert!(post.is_ok());

    // Delete the post as the creator (Alice)
    let result: Result<(), UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "delete_post",
        ("delete_test_post",),
    )
    .unwrap();

    assert!(result.is_ok());

    // Verify the post no longer exists
    let post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("delete_test_post",),
    )
    .unwrap();
    assert!(matches!(post, Err(UserPostServiceError::PostNotFound)));
}

#[test]
fn test_delete_post_unauthorized() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "delete_unauthorized_test_post",
        alice_principal,
        "video_delete_unauthorized",
        "Post to delete unauthorized",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Try to delete the post as a different user (Bob - should fail)
    let bob_principal = get_mock_user_bob_principal_id();
    let result: Result<(), UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        bob_principal,
        "delete_post",
        ("delete_unauthorized_test_post",),
    )
    .unwrap();

    assert!(result.is_err());

    // Verify the post still exists
    let post: Result<Post, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id",
        ("delete_unauthorized_test_post",),
    )
    .unwrap();
    assert!(post.is_ok());
}

#[test]
fn test_delete_post_nonexistent() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();

    let result: Result<(), UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        admin_principal,
        "delete_post",
        ("nonexistent_post",),
    )
    .unwrap();

    assert!(matches!(result, Err(UserPostServiceError::PostNotFound)));
}

#[test]
fn test_get_individual_post_details_by_id_for_user_success() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    let bob_principal = get_mock_user_bob_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "test_post_for_user",
        alice_principal,
        "video_789",
        "Test post for user details",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Get post details as Alice (creator)
    let result: Result<PostDetailsForFrontend, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id_for_user",
        ("test_post_for_user".to_string(), alice_principal),
    )
    .unwrap();

    let post_details = result.unwrap();
    assert_eq!(post_details.id, "test_post_for_user");
    assert_eq!(post_details.creator_principal, alice_principal);
    assert_eq!(post_details.video_uid, "video_789");
    assert_eq!(post_details.description, "Test post for user details");
    assert_eq!(
        post_details.hashtags,
        vec!["test".to_string(), "integration".to_string()]
    );
    assert_eq!(post_details.like_count, 0);
    assert_eq!(post_details.liked_by_me, false);

    // Get post details as Bob (different user)
    let result: Result<PostDetailsForFrontend, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        bob_principal,
        "get_individual_post_details_by_id_for_user",
        ("test_post_for_user".to_string(), bob_principal),
    )
    .unwrap();

    let post_details = result.unwrap();
    assert_eq!(post_details.id, "test_post_for_user");
    assert_eq!(post_details.creator_principal, alice_principal);
    assert_eq!(post_details.video_uid, "video_789");
    assert_eq!(post_details.description, "Test post for user details");
    assert_eq!(
        post_details.hashtags,
        vec!["test".to_string(), "integration".to_string()]
    );
    assert_eq!(post_details.like_count, 0);
    assert_eq!(post_details.liked_by_me, false);
}

#[test]
fn test_get_individual_post_details_by_id_for_user_not_found() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();

    // Try to get a non-existent post
    let result: Result<PostDetailsForFrontend, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id_for_user",
        ("non_existent_post".to_string(), alice_principal),
    )
    .unwrap();

    assert!(matches!(result, Err(UserPostServiceError::PostNotFound)));
}

#[test]
fn test_get_individual_post_details_by_id_for_user_with_likes() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_post_service_canister_id = service_canisters.user_post_service_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    let bob_principal = get_mock_user_bob_principal_id();

    // Add a post first
    let post_details = create_test_post_details(
        "liked_post",
        alice_principal,
        "video_likes",
        "Test post with likes",
    );
    let _ = add_post_as_admin(&pic, user_post_service_canister_id, post_details);

    // Bob likes the post
    let _: Result<bool, UserPostServiceError> = update(
        &pic,
        user_post_service_canister_id,
        bob_principal,
        "update_post_toggle_like_status_by_caller",
        ("liked_post",),
    )
    .unwrap();

    // Get post details as Bob
    let result: Result<PostDetailsForFrontend, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        bob_principal,
        "get_individual_post_details_by_id_for_user",
        ("liked_post".to_string(), bob_principal),
    )
    .unwrap();

    let post_details = result.unwrap();
    assert_eq!(post_details.like_count, 1);
    assert_eq!(post_details.liked_by_me, true);

    // Get post details as Alice
    let result: Result<PostDetailsForFrontend, UserPostServiceError> = query(
        &pic,
        user_post_service_canister_id,
        alice_principal,
        "get_individual_post_details_by_id_for_user",
        ("liked_post".to_string(), alice_principal),
    )
    .unwrap();

    let post_details = result.unwrap();
    assert_eq!(post_details.like_count, 1);
    assert_eq!(post_details.liked_by_me, false);
}
