use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::get_mock_user_charlie_principal_id;
use super::common::{RateLimitResult, RateLimitStatus};

#[test]
fn test_increment_request_count() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    // Increment request count
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "increment_request_count",
        (charlie_principal_id,),
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
        (charlie_principal_id,),
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
        (charlie_principal_id,),
    )
    .expect("Failed to increment request count");
    
    assert!(matches!(result, RateLimitResult::Ok(_)));
    
    // Check count increased
    let status = query::<_, Option<RateLimitStatus>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_rate_limit_status",
        (charlie_principal_id,),
    )
    .expect("Failed to get rate limit status")
    .expect("Expected status after second increment");
    
    assert_eq!(status.request_count, 2);
}