use test_utils::canister_calls::query;
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::get_mock_user_charlie_principal_id;
use super::common::GlobalRateLimitConfig;

#[test]
fn test_get_default_rate_limit_config() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    let config = query::<_, GlobalRateLimitConfig>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_default_rate_limit_config",
        (),
    )
    .expect("Failed to get default rate limit config");
    
    // Verify the config has reasonable defaults
    assert!(config.window_duration_seconds > 0);
    assert!(config.max_requests_per_window_registered > 0);
    assert!(config.max_requests_per_window_unregistered > 0);
    
    // Typically, registered users should have higher limits than unregistered
    assert!(config.max_requests_per_window_registered >= config.max_requests_per_window_unregistered);
}