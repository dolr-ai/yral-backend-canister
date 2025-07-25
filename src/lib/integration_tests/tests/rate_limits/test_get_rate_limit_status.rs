use super::common::{RateLimitResult, RateLimitStatus};
use test_utils::canister_calls::update;
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::get_mock_user_charlie_principal_id;

#[test]
fn test_get_rate_limit_status() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    // Note: We don't register the user initially to test the 'no status' case

    // Initially, there should be a default status for the principal (after the change)
    let status = update::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "default".to_string(), false),
    )
    .expect("Failed to get rate limit status");

    assert!(status.is_some(), "Expected a default status");
    let initial_status = status.unwrap();
    assert_eq!(initial_status.principal, charlie_principal_id);
    assert_eq!(initial_status.request_count, 0);
    // Unregistered users with max_requests=0 are always rate limited
    assert!(initial_status.is_limited, "Unregistered user should be rate limited with max_requests=0");

    // Test with registered user to increment request count
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "increment_request_count",
        (charlie_principal_id, "default".to_string(), true), // registered=true
    )
    .expect("Failed to increment request count");
    
    match result {
        RateLimitResult::Ok(msg) => println!("Increment result: {}", msg),
        RateLimitResult::Err(e) => panic!("Failed to increment: {}", e),
    }

    // Check status after incrementing for registered user
    let status = update::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "default".to_string(), true),
    )
    .expect("Failed to get rate limit status");

    assert!(status.is_some(), "Expected status after incrementing");
    let status = status.unwrap();
    assert_eq!(status.principal, charlie_principal_id);
    assert_eq!(status.request_count, 1);
    // Registered users have max_requests=1 by default, so they should be rate limited after 1 request
    assert!(status.is_limited, "Registered user should be rate limited after 1 request with default config");
    assert!(status.window_start > 0);
}
