use test_utils::canister_calls::update;
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};
use super::common::{RateLimitResult, register_user_for_testing};

#[test]
fn test_clear_all_rate_limits() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let global_admin = get_global_super_admin_principal_id();
    
    // Register user in user_info_service
    register_user_for_testing(&pocket_ic, &service_canisters, charlie_principal_id)
        .expect("Failed to register user");
    
    // First, increment request count for a user
    let _ = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "increment_request_count",
        (charlie_principal_id,),
    )
    .expect("Failed to increment request count");
    
    // Clear all rate limits (only admin can do this)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "clear_all_rate_limits",
        (),
    )
    .expect("Failed to clear all rate limits");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("All rate limits cleared"), "Unexpected message: {}", msg),
        RateLimitResult::Err(e) => panic!("Failed to clear rate limits: {}", e),
    }
}

#[test]
fn test_clear_all_rate_limits_unauthorized() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    // Try to clear all rate limits as a non-admin user
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "clear_all_rate_limits",
        (),
    );
    
    // Should fail due to lack of permissions
    assert!(result.is_err() || matches!(result.unwrap(), RateLimitResult::Err(_)));
}