use super::common::{RateLimitConfig, RateLimitResult};
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};

#[test]
fn test_set_principal_rate_limit() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let global_admin = get_global_super_admin_principal_id();

    // Set a custom rate limit for the principal
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_principal_rate_limit",
        (charlie_principal_id, "default".to_string(), 50u64, 300u64), // 50 requests per 300 seconds
    )
    .expect("Failed to set principal rate limit");

    match result {
        RateLimitResult::Ok(msg) => assert!(
            msg.contains("Rate limit set for principal"),
            "Unexpected message: {}",
            msg
        ),
        RateLimitResult::Err(e) => panic!("Failed to set rate limit: {}", e),
    }

    // Verify the custom config was set
    let config = query::<_, Option<RateLimitConfig>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_principal_rate_limit_config",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to get principal rate limit config")
    .expect("Expected config after setting");

    assert_eq!(config.max_requests_per_window, 50);
    assert_eq!(config.window_duration_seconds, 300);
}

#[test]
fn test_set_principal_rate_limit_unauthorized() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();

    // Try to set rate limit as a non-admin user
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "set_principal_rate_limit",
        (charlie_principal_id, "default".to_string(), 50u64, 300u64),
    );

    // Should fail due to lack of permissions
    assert!(result.is_err() || matches!(result.unwrap(), RateLimitResult::Err(_)));
}
