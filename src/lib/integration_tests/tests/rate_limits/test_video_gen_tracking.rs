use candid::Principal;
use test_utils::canister_calls::{query, update};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned,
    test_constants::get_global_super_admin_principal_id,
};

use super::common::{register_user_for_testing, RateLimitResult};

use shared_utils::canister_specific::rate_limits::{
    VideoGenRequest, VideoGenRequestKey, VideoGenRequestStatus,
};

#[test]
fn test_create_video_generation_request() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let admin = get_global_super_admin_principal_id();
    let test_user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    // Register user first
    register_user_for_testing(&pocket_ic, &service_canisters, test_user).unwrap();

    // Set a higher rate limit for the VIDEOGEN property
    let _config_result = update::<_, RateLimitResult>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "set_property_rate_limit_config",
        (
            "VIDEOGEN".to_string(),
            10u64,       // max_requests_per_window_registered
            5u64,        // max_requests_per_window_unregistered
            60u64,       // window_duration_seconds
            None::<u64>, // max_requests_per_property_all_users
            None::<u64>, // property_rate_limit_window_duration_seconds
        ),
    )
    .expect("Failed to set property rate limit config");

    // Create a video generation request
    let result = update::<_, Result<VideoGenRequestKey, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "create_video_generation_request",
        (
            test_user,
            "VEO3".to_string(),
            "Generate a video of a cat".to_string(),
            "VIDEOGEN".to_string(),
            true, // is_registered
        ),
    )
    .expect("Failed to call create_video_generation_request");

    // Should succeed with counter = 1
    assert!(result.is_ok());
    let key = result.unwrap();
    assert_eq!(key.principal, test_user);
    assert_eq!(key.counter, 1);

    // Create another request
    let result2 = update::<_, Result<VideoGenRequestKey, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "create_video_generation_request",
        (
            test_user,
            "LUMALABS".to_string(),
            "Generate a video of a dog".to_string(),
            "VIDEOGEN".to_string(),
            true,
        ),
    )
    .expect("Failed to call create_video_generation_request");

    // Should succeed with counter = 2
    if let Err(e) = &result2 {
        panic!("Second request failed: {}", e);
    }
    assert!(result2.is_ok());
    let key2 = result2.unwrap();
    assert_eq!(key2.principal, test_user);
    assert_eq!(key2.counter, 2);
}

#[test]
fn test_create_video_generation_request_rate_limited() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let admin = get_global_super_admin_principal_id();
    let test_user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    // Register user
    register_user_for_testing(&pocket_ic, &service_canisters, test_user).unwrap();

    // Set a custom rate limit for video generation
    let _result = update::<_, RateLimitResult>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "set_property_rate_limit_config",
        (
            "VIDEOGEN".to_string(),
            2u64,        // max_requests_per_window_registered
            1u64,        // max_requests_per_window_unregistered
            60u64,       // window_duration_seconds
            None::<u64>, // max_requests_per_property_all_users
            None::<u64>, // property_rate_limit_window_duration_seconds
        ),
    )
    .expect("Failed to set property rate limit config");

    // Create requests up to the limit
    for i in 0..2 {
        let result = update::<_, Result<VideoGenRequestKey, String>>(
            &pocket_ic,
            service_canisters.rate_limits_canister_id,
            admin,
            "create_video_generation_request",
            (
                test_user,
                "VEO3".to_string(),
                format!("Test prompt {}", i),
                "VIDEOGEN".to_string(),
                true,
            ),
        )
        .expect("Failed to call create_video_generation_request");

        assert!(result.is_ok(), "Request {} should succeed", i + 1);
    }

    // Third request should fail due to rate limit
    let result = update::<_, Result<VideoGenRequestKey, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "create_video_generation_request",
        (
            test_user,
            "VEO3".to_string(),
            "This should fail".to_string(),
            "VIDEOGEN".to_string(),
            true,
        ),
    )
    .expect("Failed to call create_video_generation_request");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Rate limit exceeded");
}

#[test]
fn test_get_video_generation_request() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let admin = get_global_super_admin_principal_id();
    let test_user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    // Register user
    register_user_for_testing(&pocket_ic, &service_canisters, test_user).unwrap();

    // Create a request
    let create_result = update::<_, Result<VideoGenRequestKey, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "create_video_generation_request",
        (
            test_user,
            "VEO3".to_string(),
            "Test prompt".to_string(),
            "VIDEOGEN".to_string(),
            true,
        ),
    )
    .expect("Failed to call create_video_generation_request");

    let key = create_result.unwrap();

    // Get the request
    let get_result = query::<_, Option<VideoGenRequest>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "get_video_generation_request",
        (key.clone(),),
    )
    .expect("Failed to call get_video_generation_request");

    assert!(get_result.is_some());
    let request = get_result.unwrap();
    assert_eq!(request.model_name, "VEO3");
    assert_eq!(request.prompt, "Test prompt");
    assert_eq!(request.status, VideoGenRequestStatus::Pending);
}

#[test]
fn test_update_video_generation_status() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let admin = get_global_super_admin_principal_id();
    let test_user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    // Register user
    register_user_for_testing(&pocket_ic, &service_canisters, test_user).unwrap();

    // Create a request
    let create_result = update::<_, Result<VideoGenRequestKey, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "create_video_generation_request",
        (
            test_user,
            "VEO3".to_string(),
            "Test prompt".to_string(),
            "VIDEOGEN".to_string(),
            true,
        ),
    )
    .expect("Failed to call create_video_generation_request");

    let key = create_result.unwrap();

    // Update to Processing
    let update_result = update::<_, Result<(), String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "update_video_generation_status",
        (key.clone(), VideoGenRequestStatus::Processing),
    )
    .expect("Failed to call update_video_generation_status");

    assert!(update_result.is_ok());

    // Verify the status was updated
    let get_result = query::<_, Option<VideoGenRequest>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "get_video_generation_request",
        (key.clone(),),
    )
    .expect("Failed to call get_video_generation_request");

    let request = get_result.unwrap();
    assert_eq!(request.status, VideoGenRequestStatus::Processing);

    // Update to Complete
    let result_url = "https://example.com/video.mp4".to_string();
    let update_result2 = update::<_, Result<(), String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "update_video_generation_status",
        (
            key.clone(),
            VideoGenRequestStatus::Complete(result_url.clone()),
        ),
    )
    .expect("Failed to call update_video_generation_status");

    assert!(update_result2.is_ok());

    // Verify the final status
    let get_result2 = query::<_, Option<VideoGenRequest>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "get_video_generation_request",
        (key,),
    )
    .expect("Failed to call get_video_generation_request");

    let request2 = get_result2.unwrap();
    assert_eq!(request2.status, VideoGenRequestStatus::Complete(result_url));
}

#[test]
fn test_poll_video_generation_status() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let admin = get_global_super_admin_principal_id();
    let test_user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    // Register user
    register_user_for_testing(&pocket_ic, &service_canisters, test_user).unwrap();

    // Create a request
    let create_result = update::<_, Result<VideoGenRequestKey, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "create_video_generation_request",
        (
            test_user,
            "VEO3".to_string(),
            "Test prompt".to_string(),
            "VIDEOGEN".to_string(),
            true,
        ),
    )
    .expect("Failed to call create_video_generation_request");

    let key = create_result.unwrap();

    // Poll the status
    let poll_result = query::<_, Result<VideoGenRequestStatus, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "poll_video_generation_status",
        (key.clone(),),
    )
    .expect("Failed to call poll_video_generation_status");

    assert!(poll_result.is_ok());
    assert_eq!(poll_result.unwrap(), VideoGenRequestStatus::Pending);

    // Update status and poll again
    let _update = update::<_, Result<(), String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "update_video_generation_status",
        (
            key.clone(),
            VideoGenRequestStatus::Complete("https://example.com/video.mp4".to_string()),
        ),
    )
    .expect("Failed to update status");

    let poll_result2 = query::<_, Result<VideoGenRequestStatus, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "poll_video_generation_status",
        (key,),
    )
    .expect("Failed to call poll_video_generation_status");

    assert!(poll_result2.is_ok());
    assert_eq!(
        poll_result2.unwrap(),
        VideoGenRequestStatus::Complete("https://example.com/video.mp4".to_string())
    );
}

#[test]
fn test_get_user_video_generation_requests() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let admin = get_global_super_admin_principal_id();
    let test_user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    // Register user
    register_user_for_testing(&pocket_ic, &service_canisters, test_user).unwrap();

    // Set a higher rate limit for the VIDEOGEN property
    let _config_result = update::<_, RateLimitResult>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "set_property_rate_limit_config",
        (
            "VIDEOGEN".to_string(),
            20u64, // max_requests_per_window_registered - need higher for multiple requests
            10u64, // max_requests_per_window_unregistered
            60u64, // window_duration_seconds
            None::<u64>, // max_requests_per_property_all_users
            None::<u64>, // property_rate_limit_window_duration_seconds
        ),
    )
    .expect("Failed to set property rate limit config");

    // Create multiple requests
    let mut keys = vec![];
    for i in 0..5 {
        let result = update::<_, Result<VideoGenRequestKey, String>>(
            &pocket_ic,
            service_canisters.rate_limits_canister_id,
            admin,
            "create_video_generation_request",
            (
                test_user,
                format!("MODEL_{}", i),
                format!("Prompt {}", i),
                "VIDEOGEN".to_string(),
                true,
            ),
        )
        .expect("Failed to create request");

        keys.push(result.unwrap());
    }

    // Get all requests (no cursor, default limit)
    let all_requests = query::<_, Vec<(VideoGenRequestKey, VideoGenRequest)>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "get_user_video_generation_requests",
        (test_user, None::<u64>, None::<u64>),
    )
    .expect("Failed to get user requests");

    // Should return all 5 requests in reverse order (newest first)
    assert_eq!(all_requests.len(), 5);
    assert_eq!(all_requests[0].0.counter, 5);
    assert_eq!(all_requests[4].0.counter, 1);

    // Test with limit
    let limited_requests = query::<_, Vec<(VideoGenRequestKey, VideoGenRequest)>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "get_user_video_generation_requests",
        (test_user, None::<u64>, Some(3u64)),
    )
    .expect("Failed to get user requests with limit");

    assert_eq!(limited_requests.len(), 3);
    assert_eq!(limited_requests[0].0.counter, 5);
    assert_eq!(limited_requests[2].0.counter, 3);

    // Test with cursor (start from counter 3)
    let cursor_requests = query::<_, Vec<(VideoGenRequestKey, VideoGenRequest)>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "get_user_video_generation_requests",
        (test_user, Some(3u64), Some(2u64)),
    )
    .expect("Failed to get user requests with cursor");

    assert_eq!(cursor_requests.len(), 2);
    assert_eq!(cursor_requests[0].0.counter, 3);
    assert_eq!(cursor_requests[1].0.counter, 2);
}

#[test]
fn test_video_gen_request_for_nonexistent_user() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let test_user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    // Try to get requests for a user that hasn't created any
    let requests = query::<_, Vec<(VideoGenRequestKey, VideoGenRequest)>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "get_user_video_generation_requests",
        (test_user, None::<u64>, None::<u64>),
    )
    .expect("Failed to get user requests");

    assert_eq!(requests.len(), 0);
}

#[test]
fn test_poll_nonexistent_request() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let test_user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let fake_key = VideoGenRequestKey {
        principal: test_user,
        counter: 999,
    };

    let poll_result = query::<_, Result<VideoGenRequestStatus, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        test_user,
        "poll_video_generation_status",
        (fake_key,),
    )
    .expect("Failed to call poll_video_generation_status");

    assert!(poll_result.is_err());
    assert_eq!(
        poll_result.unwrap_err(),
        "Video generation request not found"
    );
}

#[test]
fn test_decrement_video_generation_counter() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let admin = get_global_super_admin_principal_id();
    let test_user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    // Register user
    register_user_for_testing(&pocket_ic, &service_canisters, test_user).unwrap();

    // Set a low rate limit to test the decrement functionality
    let _config_result = update::<_, RateLimitResult>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "set_property_rate_limit_config",
        (
            "VIDEOGEN".to_string(),
            3u64,        // max_requests_per_window_registered
            2u64,        // max_requests_per_window_unregistered
            300u64,      // window_duration_seconds (5 minutes)
            None::<u64>, // max_requests_per_property_all_users
            None::<u64>, // property_rate_limit_window_duration_seconds
        ),
    )
    .expect("Failed to set property rate limit config");

    // Create two requests successfully
    let mut keys = vec![];
    for i in 0..2 {
        let result = update::<_, Result<VideoGenRequestKey, String>>(
            &pocket_ic,
            service_canisters.rate_limits_canister_id,
            admin,
            "create_video_generation_request",
            (
                test_user,
                "VEO3".to_string(),
                format!("Test prompt {}", i),
                "VIDEOGEN".to_string(),
                true,
            ),
        )
        .expect("Failed to call create_video_generation_request");

        assert!(result.is_ok(), "Request {} should succeed", i + 1);
        keys.push(result.unwrap());
    }

    // Update the second request to Failed status
    let failed_key = keys[1].clone();
    let update_result = update::<_, Result<(), String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "update_video_generation_status",
        (
            failed_key.clone(),
            VideoGenRequestStatus::Failed("Model error: generation failed".to_string()),
        ),
    )
    .expect("Failed to call update_video_generation_status");

    assert!(update_result.is_ok());

    // Now decrement the counter for the failed request
    let decrement_result = update::<_, Result<(), String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "decrement_video_generation_counter",
        (failed_key.clone(), "VIDEOGEN".to_string()),
    )
    .expect("Failed to call decrement_video_generation_counter");

    assert!(decrement_result.is_ok());

    // Now we should be able to create another request (since we decremented the counter)
    let result = update::<_, Result<VideoGenRequestKey, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "create_video_generation_request",
        (
            test_user,
            "VEO3".to_string(),
            "This should succeed after decrement".to_string(),
            "VIDEOGEN".to_string(),
            true,
        ),
    )
    .expect("Failed to call create_video_generation_request");

    assert!(result.is_ok(), "Request should succeed after decrement");
    let new_key = result.unwrap();
    assert_eq!(new_key.counter, 3); // Should be counter 3

    // Create another one to reach the limit again
    let result = update::<_, Result<VideoGenRequestKey, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "create_video_generation_request",
        (
            test_user,
            "VEO3".to_string(),
            "This should also succeed after decrement".to_string(),
            "VIDEOGEN".to_string(),
            true,
        ),
    )
    .expect("Failed to call create_video_generation_request");

    assert!(result.is_ok(), "Fourth request should succeed after decrement");
    assert_eq!(result.unwrap().counter, 4);

    // Now try to create one more - this should fail (we're at the limit)
    let result = update::<_, Result<VideoGenRequestKey, String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "create_video_generation_request",
        (
            test_user,
            "VEO3".to_string(),
            "This should fail".to_string(),
            "VIDEOGEN".to_string(),
            true,
        ),
    )
    .expect("Failed to call create_video_generation_request");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Rate limit exceeded");

    // Test that decrement only works for failed requests
    let non_failed_key = keys[0].clone();
    let decrement_result = update::<_, Result<(), String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "decrement_video_generation_counter",
        (non_failed_key, "VIDEOGEN".to_string()),
    )
    .expect("Failed to call decrement_video_generation_counter");

    assert!(decrement_result.is_err());
    assert_eq!(
        decrement_result.unwrap_err(),
        "Can only decrement counter for failed requests"
    );

    // Test that decrement fails for non-existent request
    let fake_key = VideoGenRequestKey {
        principal: test_user,
        counter: 999,
    };
    let decrement_result = update::<_, Result<(), String>>(
        &pocket_ic,
        service_canisters.rate_limits_canister_id,
        admin,
        "decrement_video_generation_counter",
        (fake_key, "VIDEOGEN".to_string()),
    )
    .expect("Failed to call decrement_video_generation_counter");

    assert!(decrement_result.is_err());
    assert_eq!(
        decrement_result.unwrap_err(),
        "Video generation request not found"
    );
}
