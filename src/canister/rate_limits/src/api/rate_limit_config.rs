use candid::Principal;
use ic_cdk_macros::{query, update};
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::{SharedGlobalRateLimitConfig, RateLimitConfig, RateLimitResult, CANISTER_DATA};

/// Get default configuration
#[query]
pub fn get_default_rate_limit_config() -> SharedGlobalRateLimitConfig {
    CANISTER_DATA.with(|data| {
        let data = data.borrow();
        data.default_config.0.clone()
    })
}

/// Update default configuration (admin only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub fn update_default_rate_limit_config(
    max_requests_per_window_registered: u64,
    max_requests_per_window_unregistered: u64,
    window_duration_seconds: u64,
) -> RateLimitResult {
    if max_requests_per_window_registered == 0
        || max_requests_per_window_unregistered == 0
        || window_duration_seconds == 0
    {
        return RateLimitResult::Err(
            "Invalid configuration: values must be greater than 0".to_string(),
        );
    }

    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.default_config.max_requests_per_window_registered = max_requests_per_window_registered;
        data.default_config.max_requests_per_window_unregistered =
            max_requests_per_window_unregistered;
        data.default_config.window_duration_seconds = window_duration_seconds;
        RateLimitResult::Ok("Default configuration updated".to_string())
    })
}

/// Get configuration for a specific principal
#[query]
pub fn get_principal_rate_limit_config(principal: Principal) -> Option<RateLimitConfig> {
    CANISTER_DATA.with(|data| {
        let data = data.borrow();
        data.get_principal_config(&principal)
    })
}