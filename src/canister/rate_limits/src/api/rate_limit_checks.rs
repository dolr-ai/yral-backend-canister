use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::{CANISTER_DATA, RateLimitResult, RateLimitStatus};

/// Guard function to check if caller is allowed to increment request count
fn is_caller_allowed_to_increment(principal: &Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();

    if caller == *principal || is_caller_controller_or_global_admin().is_ok() {
        return Ok(());
    }

    Err(
        "Unauthorized: Only admin, controller, or the principal itself can increment request count"
            .to_string(),
    )
}

/// Check if a principal has exceeded rate limits for a specific property
/// using update call for consistent reads
#[update]
pub async fn check_rate_limit(
    principal: Principal,
    property: String,
    is_registered: bool,
) -> RateLimitResult {
    CANISTER_DATA.with(|data| {
        let data = data.borrow();
        if data.is_rate_limited_with_property(&principal, &property, is_registered) {
            RateLimitResult::Err("Rate limit exceeded".to_string())
        } else {
            RateLimitResult::Ok("Within rate limit".to_string())
        }
    })
}

/// Increment the request count for a principal on a specific property
#[update]
pub async fn increment_request_count(
    principal: Principal,
    property: String,
    is_registered: bool,
) -> RateLimitResult {
    // Check if caller is authorized
    if let Err(e) = is_caller_allowed_to_increment(&principal) {
        return RateLimitResult::Err(e);
    }

    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        if data.is_rate_limited_with_property(&principal, &property, is_registered) {
            RateLimitResult::Err("Rate limit exceeded".to_string())
        } else {
            data.increment_request_with_property(&principal, &property);
            RateLimitResult::Ok("Request count incremented".to_string())
        }
    })
}

/// Increment the request count for a principal on a specific property with payment support
#[update]
pub async fn increment_request_count_v1(
    principal: Principal,
    property: String,
    is_registered: bool,
    is_paid: bool,
    payment_amount: Option<String>,
) -> RateLimitResult {
    // Check if caller is authorized
    if let Err(e) = is_caller_allowed_to_increment(&principal) {
        return RateLimitResult::Err(e);
    }

    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();

        // If this is a paid request, check property limit and increment property counter only
        if is_paid {
            // Check property-wide rate limit for paid requests
            if data.is_property_daily_rate_limited_for_paid(&property) {
                return RateLimitResult::Err(
                    "Property rate limit exceeded for paid request".to_string(),
                );
            }

            // Increment only the property-wide counter
            data.increment_paid_request_property_only(&property);

            RateLimitResult::Ok(format!(
                "Paid request processed. Amount: {:?}",
                payment_amount
            ))
        } else {
            // For unpaid requests, use the existing logic
            if data.is_rate_limited_with_property(&principal, &property, is_registered) {
                RateLimitResult::Err("Rate limit exceeded".to_string())
            } else {
                data.increment_request_with_property(&principal, &property);
                RateLimitResult::Ok("Request count incremented".to_string())
            }
        }
    })
}

/// Get the current rate limit status for a principal on a specific property
#[update]
pub async fn get_rate_limit_status(
    principal: Principal,
    property: String,
    is_registered: bool,
) -> Option<RateLimitStatus> {
    CANISTER_DATA.with(|data| {
        let data = data.borrow();
        data.get_rate_limit_status_with_property(&principal, &property, is_registered)
    })
}
