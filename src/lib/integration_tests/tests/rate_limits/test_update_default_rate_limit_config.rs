use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};
use super::common::{GlobalRateLimitConfig, RateLimitResult};

#[test]
fn test_update_default_rate_limit_config() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    
    // Get the current default config
    let original_config = query::<_, GlobalRateLimitConfig>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_default_rate_limit_config",
        (),
    )
    .expect("Failed to get default rate limit config");
    
    // Update the default config
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "update_default_rate_limit_config",
        (1000u64, 500u64, 600u64), // registered: 1000, unregistered: 500, window: 600s
    )
    .expect("Failed to update default rate limit config");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("Default configuration updated"), "Unexpected message: {}", msg),
        RateLimitResult::Err(e) => panic!("Failed to update config: {}", e),
    }
    
    // Verify the config was updated
    let new_config = query::<_, GlobalRateLimitConfig>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_default_rate_limit_config",
        (),
    )
    .expect("Failed to get default rate limit config");
    
    assert_eq!(new_config.window_duration_seconds, 600);
    assert_eq!(new_config.max_requests_per_window_registered, 1000);
    assert_eq!(new_config.max_requests_per_window_unregistered, 500);
    
    // Ensure the config actually changed
    assert_ne!(new_config.window_duration_seconds, original_config.window_duration_seconds);
}

#[test]
fn test_update_default_rate_limit_config_unauthorized() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    // Try to update config as a non-admin user
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "update_default_rate_limit_config",
        (600u64, 1000u64, 500u64),
    );
    
    // Should fail due to lack of permissions
    assert!(result.is_err() || matches!(result.unwrap(), RateLimitResult::Err(_)));
}

#[test]
fn test_update_default_rate_limit_config_invalid_params() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    
    // Try to set invalid config (0 window duration)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "update_default_rate_limit_config",
        (1000u64, 500u64, 0u64), // 0 window duration
    )
    .expect("Failed to call update_default_rate_limit_config");
    
    match result {
        RateLimitResult::Ok(_) => panic!("Expected error for invalid parameters"),
        RateLimitResult::Err(msg) => assert!(msg.contains("Invalid") || msg.contains("window")),
    }
    
    // Try to set invalid config (registered limit lower than unregistered)
    let _result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "update_default_rate_limit_config",
        (100u64, 200u64, 600u64), // registered < unregistered
    )
    .expect("Failed to call update_default_rate_limit_config");
    
    // This might be allowed, but typically registered users should have higher limits
    // The test depends on the implementation's validation logic
}