use super::common::{RateLimitResult, RateLimitStatus};
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::get_mock_user_charlie_principal_id;

#[test]
fn test_get_rate_limit_status() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    // Note: We don't register the user initially to test the 'no status' case

    // Initially, there should be no status for the principal
    let status = query::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to get rate limit status");

    assert!(status.is_none(), "Expected no status initially");

    // Increment request count to create a status
    let _ = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "increment_request_count",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to increment request count");

    // Now there should be a status
    let status = query::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to get rate limit status");

    assert!(status.is_some(), "Expected status after incrementing");
    let status = status.unwrap();
    assert_eq!(status.principal, charlie_principal_id);
    assert_eq!(status.request_count, 1);
    assert!(!status.is_limited);
    assert!(status.window_start > 0);
}
