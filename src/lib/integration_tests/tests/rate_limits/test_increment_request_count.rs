use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::get_mock_user_charlie_principal_id;
use super::common::{RateLimitResult, RateLimitStatus, register_user_for_testing, GlobalRateLimitConfig, get_session_type_for_principal};

#[test]
fn test_increment_request_count() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    // Register user in user_info_service
    register_user_for_testing(&pocket_ic, &service_canisters, charlie_principal_id)
        .expect("Failed to register user");
    
    // Check the default rate limit config
    let default_config = query::<_, GlobalRateLimitConfig>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_default_rate_limit_config",
        (),
    )
    .expect("Failed to get default rate limit config");
    
    println!("Default config - registered: {}, unregistered: {}, window: {}", 
        default_config.max_requests_per_window_registered,
        default_config.max_requests_per_window_unregistered,
        default_config.window_duration_seconds);
    
    // Check the session type for the user
    let session_type = get_session_type_for_principal(&pocket_ic, &service_canisters, charlie_principal_id)
        .expect("Failed to get session type");
    println!("Session type for charlie: {:?}", session_type);
    
    // Set a custom rate limit for this user that allows multiple requests
    let admin_principal = test_utils::setup::test_constants::get_global_super_admin_principal_id();
    let set_result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        admin_principal,
        "set_principal_rate_limit",
        (charlie_principal_id, "default".to_string(), 10u64, 86400u64), // Allow 10 requests per day
    )
    .expect("Failed to set principal rate limit");
    
    match set_result {
        RateLimitResult::Ok(msg) => println!("Set rate limit: {}", msg),
        RateLimitResult::Err(e) => panic!("Failed to set rate limit: {}", e),
    }
    
    // Increment request count
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "increment_request_count",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to increment request count");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Request count incremented")),
        RateLimitResult::Err(e) => panic!("Failed to increment request count: {}", e),
    }
    
    // Check the status to verify increment
    let status = query::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to get rate limit status")
    .expect("Expected status after increment");
    
    assert_eq!(status.request_count, 1);
    
    // Increment again
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "increment_request_count",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to increment request count");
    
    match result {
        RateLimitResult::Ok(_msg) => {},
        RateLimitResult::Err(e) => panic!("Second increment failed: {}", e),
    }
    
    // Check count increased
    let status = query::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to get rate limit status")
    .expect("Expected status after second increment");
    
    assert_eq!(status.request_count, 2);
}