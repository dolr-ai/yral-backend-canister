use test_utils::canister_calls::update;
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};
use super::common::RateLimitResult;

#[test]
fn test_check_rate_limit() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    // Check rate limit for a user who hasn't made any requests
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "check_rate_limit",
        (charlie_principal_id,),
    )
    .expect("Failed to check rate limit");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Rate limit check passed")),
        RateLimitResult::Err(e) => panic!("Rate limit check failed: {}", e),
    }
}

#[test]
fn test_check_rate_limit_exceeds_limit() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let global_admin = get_global_super_admin_principal_id();
    
    // First, set a very low rate limit for testing
    let _ = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_principal_rate_limit",
        (charlie_principal_id, 60u64, 2u64), // 2 requests per 60 seconds
    )
    .expect("Failed to set rate limit");
    
    // Make requests up to the limit
    for _ in 0..2 {
        let _ = update::<_, RateLimitResult>(
            &pocket_ic,
            rate_limits_canister,
            charlie_principal_id,
            "increment_request_count",
            (charlie_principal_id,),
        )
        .expect("Failed to increment request count");
    }
    
    // Check rate limit - should now fail
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "check_rate_limit",
        (charlie_principal_id,),
    )
    .expect("Failed to check rate limit");
    
    match result {
        RateLimitResult::Ok(_) => panic!("Expected rate limit to be exceeded"),
        RateLimitResult::Err(msg) => assert!(msg.contains("Rate limit exceeded")),
    }
}