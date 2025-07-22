use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};
use super::common::{RateLimitResult, RateLimitStatus, register_user_for_testing};

#[test]
fn test_reset_rate_limit() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let global_admin = get_global_super_admin_principal_id();
    
    // Register user in user_info_service
    register_user_for_testing(&pocket_ic, &service_canisters, charlie_principal_id)
        .expect("Failed to register user");
    
    // Set a custom rate limit for this user that allows multiple requests
    let set_result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_principal_rate_limit",
        (charlie_principal_id, "default".to_string(), 10u64, 86400u64), // Allow 10 requests per day
    )
    .expect("Failed to set principal rate limit");
    
    match set_result {
        RateLimitResult::Ok(_) => {},
        RateLimitResult::Err(e) => panic!("Failed to set rate limit: {}", e),
    }
    
    // First, increment request count a few times
    for i in 0..3 {
        let result = update::<_, RateLimitResult>(
            &pocket_ic,
            rate_limits_canister,
            charlie_principal_id,
            "increment_request_count",
            (charlie_principal_id, "default".to_string()),
        )
        .expect("Failed to increment request count");
        
        match result {
            RateLimitResult::Ok(_) => {},
            RateLimitResult::Err(e) => panic!("Failed to increment request count {}: {}", i + 1, e),
        }
    }
    
    // Verify the count is 3
    let status = query::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to get rate limit status")
    .expect("Expected status after increments");
    
    assert_eq!(status.request_count, 3);
    
    // Reset the rate limit (only admin can do this)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "reset_rate_limit",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to reset rate limit");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Rate limit reset"), "Unexpected message: {}", msg),
        RateLimitResult::Err(e) => panic!("Failed to reset rate limit: {}", e),
    }
    
    // Verify the rate limit is reset
    let status = query::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "default".to_string()),
    );
    
    // After reset, the status might be None or have count 0
    match status {
        Ok(Some(s)) => assert_eq!(s.request_count, 0),
        Ok(None) => {}, // This is also acceptable
        Err(e) => panic!("Failed to get status: {:?}", e),
    }
}

#[test]
fn test_reset_rate_limit_unauthorized() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    // Try to reset rate limit as a non-admin user
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "reset_rate_limit",
        (charlie_principal_id, "default".to_string()),
    );
    
    // Should fail due to lack of permissions
    assert!(result.is_err() || matches!(result.unwrap(), RateLimitResult::Err(_)));
}