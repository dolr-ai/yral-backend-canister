use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};
use super::common::{PropertyRateLimitConfig, RateLimitConfig, RateLimitResult, RateLimitStatus, register_user_for_testing};

#[test]
fn test_property_based_rate_limiting() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let global_admin = get_global_super_admin_principal_id();
    
    // Register user in user_info_service
    register_user_for_testing(&pocket_ic, &service_canisters, charlie_principal_id)
        .expect("Failed to register user");
    
    // Set different rate limits for different properties
    let _ = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_property_rate_limit_config",
        ("upload_video".to_string(), 2u64, 1u64, 60u64), // Registered: 2/min, Unregistered: 1/min
    )
    .expect("Failed to set property config");
    
    let _ = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_property_rate_limit_config",
        ("create_post".to_string(), 10u64, 5u64, 60u64), // Registered: 10/min, Unregistered: 5/min
    )
    .expect("Failed to set property config");
    
    // Test that different properties have independent rate limits
    
    // Use up the upload_video limit (1 request for unregistered users)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "increment_request_count",
        (charlie_principal_id, "upload_video".to_string()),
    )
    .expect("Failed to increment request count");
    
    match result {
        RateLimitResult::Ok(_) => {},
        RateLimitResult::Err(e) => panic!("Failed to increment upload_video request 1: {}", e),
    }
    
    // Second upload_video request should fail (unregistered users only get 1)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "increment_request_count",
        (charlie_principal_id, "upload_video".to_string()),
    )
    .expect("Failed to call increment_request_count");
    
    match result {
        RateLimitResult::Ok(_) => panic!("Expected upload_video to be rate limited"),
        RateLimitResult::Err(msg) => assert!(msg.contains("Rate limit exceeded"), "Unexpected error: {}", msg),
    }
    
    // But create_post should still work (different property, different limit)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "increment_request_count",
        (charlie_principal_id, "create_post".to_string()),
    )
    .expect("Failed to increment create_post request");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Request count incremented")),
        RateLimitResult::Err(e) => panic!("create_post should not be rate limited: {}", e),
    }
    
    // Verify the status of each property
    let upload_status = query::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "upload_video".to_string(), true),
    )
    .expect("Failed to get upload_video status")
    .expect("Expected upload_video status");
    
    assert_eq!(upload_status.request_count, 1);
    // For unregistered users with a limit of 1, having 1 request means we're at the limit
    // The is_limited flag shows if we've exceeded the limit, not if we're at it
    assert!(!upload_status.is_limited); // Not yet exceeded, just at the limit
    
    let post_status = query::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "create_post".to_string(), true),
    )
    .expect("Failed to get create_post status")
    .expect("Expected create_post status");
    
    assert_eq!(post_status.request_count, 1);
    assert!(!post_status.is_limited);
}

#[test]
fn test_property_config_management() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    
    // Set a property config
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_property_rate_limit_config",
        ("api.upload".to_string(), 5u64, 2u64, 3600u64), // Registered: 5/hour, Unregistered: 2/hour
    )
    .expect("Failed to set property config");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Property rate limit config set")),
        RateLimitResult::Err(e) => panic!("Failed to set property config: {}", e),
    }
    
    // Get the property config
    let config = query::<_, Option<PropertyRateLimitConfig>>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_property_rate_limit_config",
        ("api.upload".to_string(),),
    )
    .expect("Failed to get property config")
    .expect("Expected property config");
    
    assert_eq!(config.property, "api.upload");
    assert_eq!(config.max_requests_per_window_registered, 5);
    assert_eq!(config.max_requests_per_window_unregistered, 2);
    assert_eq!(config.window_duration_seconds, 3600);
    
    // Get all property configs
    let all_configs = query::<_, Vec<PropertyRateLimitConfig>>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_property_rate_limit_configs",
        (),
    )
    .expect("Failed to get all property configs");
    
    assert!(!all_configs.is_empty());
    assert!(all_configs.iter().any(|c| c.property == "api.upload"));
    
    // Remove the property config
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "remove_property_rate_limit_config",
        ("api.upload".to_string(),),
    )
    .expect("Failed to remove property config");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Property rate limit config removed")),
        RateLimitResult::Err(e) => panic!("Failed to remove property config: {}", e),
    }
    
    // Verify it's gone
    let config = query::<_, Option<PropertyRateLimitConfig>>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_property_rate_limit_config",
        ("api.upload".to_string(),),
    )
    .expect("Failed to get property config");
    
    assert!(config.is_none(), "Property config should be removed");
}

#[test]
fn test_principal_property_rate_limits() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let global_admin = get_global_super_admin_principal_id();
    
    // Set a custom rate limit for a specific principal+property combination
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_principal_rate_limit",
        (charlie_principal_id, "special_action".to_string(), 100u64, 3600u64), // 100 requests per hour
    )
    .expect("Failed to set principal property rate limit");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Rate limit set for principal")),
        RateLimitResult::Err(e) => panic!("Failed to set principal property rate limit: {}", e),
    }
    
    // Verify the custom config was set
    let config = query::<_, Option<RateLimitConfig>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_principal_rate_limit_config",
        (charlie_principal_id, "special_action".to_string()),
    )
    .expect("Failed to get principal rate limit config")
    .expect("Expected config after setting");
    
    assert_eq!(config.max_requests_per_window, 100);
    assert_eq!(config.window_duration_seconds, 3600);
    
    // Reset rate limit for this specific property
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "reset_rate_limit",
        (charlie_principal_id, "special_action".to_string()),
    )
    .expect("Failed to reset rate limit");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Rate limit reset")),
        RateLimitResult::Err(e) => panic!("Failed to reset rate limit: {}", e),
    }
}