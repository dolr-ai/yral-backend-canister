use candid::Principal;
use ic_cdk_macros::{query, update};
use shared_utils::{
    canister_specific::rate_limits::PropertyRateLimitConfig,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use crate::{CANISTER_DATA, RateLimitConfig, RateLimitResult};

/// Reset rate limit for a specific principal on a specific property (admin only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub fn reset_rate_limit(principal: Principal, property: String) -> RateLimitResult {
    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.reset_rate_limit_with_property(&principal, &property);
        RateLimitResult::Ok(format!(
            "Rate limit reset for principal {} on property {}",
            principal, property
        ))
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

/// Set custom rate limit for a specific principal on a specific property (admin only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub fn set_principal_rate_limit(
    principal: Principal,
    property: String,
    max_requests_per_window: u64,
    window_duration_seconds: u64,
) -> RateLimitResult {
    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        let config = RateLimitConfig {
            max_requests_per_window,
            window_duration_seconds,
        };
        data.set_principal_property_rate_limit(&principal, &property, config);
        RateLimitResult::Ok(format!(
            "Rate limit set for principal {} on property {}",
            principal, property
        ))
    })
}

/// Set default rate limit configuration for a specific property (admin only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub fn set_property_rate_limit_config(
    property: String,
    max_requests_per_window_registered: u64,
    max_requests_per_window_unregistered: u64,
    window_duration_seconds: u64,
) -> RateLimitResult {
    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        let config = PropertyRateLimitConfig {
            property: property.clone(),
            max_requests_per_window_registered,
            max_requests_per_window_unregistered,
            window_duration_seconds,
        };
        data.set_property_config(config);
        RateLimitResult::Ok(format!("Property rate limit config set for: {}", property))
    })
}

/// Remove property rate limit configuration (admin only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub fn remove_property_rate_limit_config(property: String) -> RateLimitResult {
    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.remove_property_config(&property);
        RateLimitResult::Ok(format!(
            "Property rate limit config removed for: {}",
            property
        ))
    })
}

/// Reset all rate limits for a specific principal across all properties (admin only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub fn reset_all_principal_rate_limits(principal: Principal) -> RateLimitResult {
    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.reset_all_principal_rate_limits(&principal);
        RateLimitResult::Ok(format!(
            "All rate limits reset for principal: {}",
            principal
        ))
    })
}

/// Get all property rate limit configurations
#[query]
pub fn get_property_rate_limit_configs() -> Vec<PropertyRateLimitConfig> {
    CANISTER_DATA.with(|data| data.borrow().get_all_property_configs())
}

/// Get property rate limit configuration for a specific property
#[query]
pub fn get_property_rate_limit_config(property: String) -> Option<PropertyRateLimitConfig> {
    CANISTER_DATA.with(|data| data.borrow().get_property_config(&property))
}
