use candid::Principal;
use shared_utils::canister_specific::rate_limits::RateLimitResult;
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};

use super::common::register_user_for_testing;

#[test]
fn test_property_wide_rate_limit() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    
    let property = "test_property";
    let principal1 = Principal::from_text("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
    let principal2 = Principal::from_text("2vxsx-fae").unwrap();
    
    // Register users
    register_user_for_testing(&pocket_ic, &service_canisters, principal1)
        .expect("Failed to register principal1");
    register_user_for_testing(&pocket_ic, &service_canisters, principal2)
        .expect("Failed to register principal2");
    
    // Configure property with a limit of 3 requests across all users
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_property_rate_limit_config",
        (
            property,
            10u64,  // max_requests_per_window_registered
            5u64,   // max_requests_per_window_unregistered
            3600u64,  // window_duration_seconds
            Some(3u64),  // max_requests_per_property_all_users
            Some(86400u64),  // property_rate_limit_window_duration_seconds (24 hours)
        ),
    )
    .expect("Failed to set property config");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Property rate limit config set")),
        RateLimitResult::Err(e) => panic!("Failed to set property config: {}", e),
    }
    
    // First 3 requests should succeed (2 from principal1, 1 from principal2)
    // Request 1 from principal1
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal1,
        "increment_request_count",
        (principal1, property, false),
    )
    .expect("Failed to call increment_request_count");
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Request count incremented")),
        RateLimitResult::Err(e) => panic!("Request 1 failed: {}", e),
    }
    
    // Request 2 from principal1
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal1,
        "increment_request_count",
        (principal1, property, false),
    )
    .expect("Failed to call increment_request_count");
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Request count incremented")),
        RateLimitResult::Err(e) => panic!("Request 2 failed: {}", e),
    }
    
    // Request 3 from principal2
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal2,
        "increment_request_count",
        (principal2, property, false),
    )
    .expect("Failed to call increment_request_count");
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Request count incremented")),
        RateLimitResult::Err(e) => panic!("Request 3 failed: {}", e),
    }
    
    // 4th request should be rate limited (property-wide limit reached)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal1,
        "check_rate_limit",
        (principal1, property, false),
    )
    .expect("Failed to call check_rate_limit");
    match result {
        RateLimitResult::Ok(_) => panic!("Expected rate limit to be exceeded"),
        RateLimitResult::Err(msg) => assert!(msg.contains("Rate limit exceeded")),
    }
    
    // Check that the property usage is 3
    let usage: u64 = query(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_property_daily_usage",
        (property,),
    )
    .expect("Failed to get property daily usage");
    assert_eq!(usage, 3);
}

#[test]
fn test_property_wide_rate_limit_window_reset() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    
    let property = "test_property";
    let principal = get_mock_user_charlie_principal_id();
    
    // Register user
    register_user_for_testing(&pocket_ic, &service_canisters, principal)
        .expect("Failed to register user");
    
    // Configure property with a limit of 2 requests with a very short window
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_property_rate_limit_config",
        (
            property,
            10u64,  // max_requests_per_window_registered
            5u64,   // max_requests_per_window_unregistered
            3600u64,  // window_duration_seconds
            Some(2u64),  // max_requests_per_property_all_users
            Some(1u64),  // property_rate_limit_window_duration_seconds (1 second window)
        ),
    )
    .expect("Failed to set property config");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Property rate limit config set")),
        RateLimitResult::Err(e) => panic!("Failed to set property config: {}", e),
    }
    
    // Use up the limit
    for _ in 0..2 {
        let result = update::<_, RateLimitResult>(
            &pocket_ic,
            rate_limits_canister,
            principal,
            "increment_request_count",
            (principal, property, false),
        )
        .expect("Failed to call increment_request_count");
        match result {
            RateLimitResult::Ok(msg) => assert!(msg.contains("Request count incremented")),
            RateLimitResult::Err(e) => panic!("Failed to increment: {}", e),
        }
    }
    
    // Should be rate limited now
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal,
        "check_rate_limit",
        (principal, property, false),
    )
    .expect("Failed to call check_rate_limit");
    match result {
        RateLimitResult::Ok(_) => panic!("Expected rate limit to be exceeded"),
        RateLimitResult::Err(msg) => assert!(msg.contains("Rate limit exceeded")),
    }
    
    // Reset the property limit
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "reset_property_daily_limit",
        (property,),
    )
    .expect("Failed to call reset_property_daily_limit");
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Property-wide rate limit counter reset")),
        RateLimitResult::Err(e) => panic!("Failed to reset: {}", e),
    }
    
    // Should be able to make requests again
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal,
        "check_rate_limit",
        (principal, property, false),
    )
    .expect("Failed to call check_rate_limit");
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Within rate limit")),
        RateLimitResult::Err(e) => panic!("Should not be rate limited after reset: {}", e),
    }
    
    // Usage should be 0
    let usage: u64 = query(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_property_daily_usage",
        (property,),
    )
    .expect("Failed to get property daily usage");
    assert_eq!(usage, 0);
}

#[test]
fn test_property_wide_limit_not_configured() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    
    let property = "test_property";
    let principal = get_mock_user_charlie_principal_id();
    
    // Register user
    register_user_for_testing(&pocket_ic, &service_canisters, principal)
        .expect("Failed to register user");
    
    // Configure property without property-wide limit
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_property_rate_limit_config",
        (
            property,
            2u64,   // max_requests_per_window_registered
            1u64,   // max_requests_per_window_unregistered
            3600u64,  // window_duration_seconds
            None::<u64>,  // max_requests_per_property_all_users
            None::<u64>,  // property_rate_limit_window_duration_seconds
        ),
    )
    .expect("Failed to set property config");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Property rate limit config set")),
        RateLimitResult::Err(e) => panic!("Failed to set property config: {}", e),
    }
    
    // Should only be limited by per-principal limits
    // Use registered=true since we configured 2 requests for registered users
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal,
        "increment_request_count",
        (principal, property, true),
    )
    .expect("Failed to call increment_request_count");
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Request count incremented")),
        RateLimitResult::Err(e) => panic!("Failed to increment: {}", e),
    }
    
    // Check rate limit should still be within limits (1 out of 2 for registered users)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal,
        "check_rate_limit",
        (principal, property, true),
    )
    .expect("Failed to call check_rate_limit");
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Within rate limit")),
        RateLimitResult::Err(e) => panic!("Should not be rate limited: {}", e),
    }
    
    // Property usage should be 0 since property-wide limit is not configured
    let usage: u64 = query(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_property_daily_usage",
        (property,),
    )
    .expect("Failed to get property daily usage");
    assert_eq!(usage, 0);
}

#[test]
fn test_both_limits_enforced() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    
    let property = "test_property";
    let principal = get_mock_user_charlie_principal_id();
    
    // Register user
    register_user_for_testing(&pocket_ic, &service_canisters, principal)
        .expect("Failed to register user");
    
    // Configure with both per-principal and property-wide limits
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_property_rate_limit_config",
        (
            property,
            10u64,  // max_requests_per_window_registered
            2u64,   // max_requests_per_window_unregistered (Per-principal limit: 2)
            3600u64,  // window_duration_seconds
            Some(5u64),  // max_requests_per_property_all_users (Property-wide limit: 5)
            Some(86400u64),  // property_rate_limit_window_duration_seconds
        ),
    )
    .expect("Failed to set property config");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Property rate limit config set")),
        RateLimitResult::Err(e) => panic!("Failed to set property config: {}", e),
    }
    
    // Principal should hit their personal limit before property-wide limit
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal,
        "increment_request_count",
        (principal, property, false),
    )
    .expect("Failed to call increment_request_count");
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Request count incremented")),
        RateLimitResult::Err(e) => panic!("First request failed: {}", e),
    }
    
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal,
        "increment_request_count",
        (principal, property, false),
    )
    .expect("Failed to call increment_request_count");
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Request count incremented")),
        RateLimitResult::Err(e) => panic!("Second request failed: {}", e),
    }
    
    // Principal has hit their personal limit (2 requests)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        principal,
        "check_rate_limit",
        (principal, property, false),
    )
    .expect("Failed to call check_rate_limit");
    match result {
        RateLimitResult::Ok(_) => panic!("Expected rate limit to be exceeded"),
        RateLimitResult::Err(msg) => assert!(msg.contains("Rate limit exceeded")),
    }
    
    // Property-wide usage is only 2 (out of 5 allowed)
    let usage: u64 = query(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_property_daily_usage",
        (property,),
    )
    .expect("Failed to get property daily usage");
    assert_eq!(usage, 2);
}