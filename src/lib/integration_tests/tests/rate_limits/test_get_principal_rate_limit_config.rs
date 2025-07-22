use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};
use super::common::{RateLimitConfig, RateLimitResult};

#[test]
fn test_get_principal_rate_limit_config() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let global_admin = get_global_super_admin_principal_id();
    
    // Initially, there should be no custom config for the principal
    let config = query::<_, Option<RateLimitConfig>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_principal_rate_limit_config",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to get principal rate limit config");
    
    assert!(config.is_none(), "Expected no custom config initially");
    
    // Set a custom rate limit for the principal
    let _ = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "set_principal_rate_limit",
        (charlie_principal_id, "default".to_string(), 100u64, 120u64), // 100 requests per 120 seconds
    )
    .expect("Failed to set principal rate limit");
    
    // Now there should be a custom config
    let config = query::<_, Option<RateLimitConfig>>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_principal_rate_limit_config",
        (charlie_principal_id, "default".to_string()),
    )
    .expect("Failed to get principal rate limit config");
    
    assert!(config.is_some(), "Expected custom config after setting");
    let config = config.unwrap();
    assert_eq!(config.window_duration_seconds, 120);
    assert_eq!(config.max_requests_per_window, 100);
}