use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::{RateLimitConfig, RateLimitResult, CANISTER_DATA};

/// Reset rate limits for a specific principal (admin only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub fn reset_rate_limit(principal: Principal) -> RateLimitResult {
    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.reset_rate_limit(&principal);
        RateLimitResult::Ok("Rate limit reset".to_string())
    })
}

/// Clear all rate limits (admin only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub fn clear_all_rate_limits() -> RateLimitResult {
    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.clear_all_rate_limits();
        RateLimitResult::Ok("All rate limits cleared".to_string())
    })
}

/// Set custom rate limit for a specific principal (admin only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub fn set_principal_rate_limit(
    principal: Principal,
    max_requests_per_window: u64,
    window_duration_seconds: u64,
) -> RateLimitResult {
    if max_requests_per_window == 0 || window_duration_seconds == 0 {
        return RateLimitResult::Err(
            "Invalid configuration: values must be greater than 0".to_string(),
        );
    }

    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        let config = RateLimitConfig {
            max_requests_per_window,
            window_duration_seconds,
        };
        data.set_principal_rate_limit(&principal, config);
        RateLimitResult::Ok(format!("Rate limit set for principal: {}", principal))
    })
}