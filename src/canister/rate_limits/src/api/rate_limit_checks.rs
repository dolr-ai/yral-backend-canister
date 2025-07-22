use candid::Principal;
use ic_cdk_macros::{query, update};
use shared_utils::canister_specific::individual_user_template::types::session::SessionType;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::{CANISTER_DATA, RateLimitResult, RateLimitStatus, utils};

/// Guard function to check if caller is allowed to increment request count
fn is_caller_allowed_to_increment(principal: &Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();

    // Allow if caller is controller or global admin
    if is_caller_controller_or_global_admin().is_ok() {
        return Ok(());
    }

    // Allow if caller is the same as the principal being rate limited
    if caller == *principal {
        return Ok(());
    }

    Err(
        "Unauthorized: Only admin, controller, or the principal itself can increment request count"
            .to_string(),
    )
}

/// Check if a principal has exceeded rate limits for a specific property
#[query]
pub async fn check_rate_limit(principal: Principal, property: String) -> RateLimitResult {
    // Get user_info_canister from CANISTER_DATA
    let user_info_canister = CANISTER_DATA.with(|data| data.borrow().user_info_canister);

    // Get session type from user_info_service
    let is_registered =
        match utils::get_session_type_for_principal(user_info_canister, principal).await {
            Ok(session_type) => match session_type {
                SessionType::RegisteredSession => true,
                SessionType::AnonymousSession => false,
            },
            Err(e) => {
                // If we can't determine session type, treat as anonymous/unregistered for safety
                ic_cdk::println!("Failed to get session type: {}", e);
                false
            }
        };

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
pub async fn increment_request_count(principal: Principal, property: String) -> RateLimitResult {
    // Check if caller is authorized
    if let Err(e) = is_caller_allowed_to_increment(&principal) {
        return RateLimitResult::Err(e);
    }

    // Get user_info_canister from CANISTER_DATA
    let user_info_canister = CANISTER_DATA.with(|data| data.borrow().user_info_canister);

    // Get session type from user_info_service
    let is_registered =
        match utils::get_session_type_for_principal(user_info_canister, principal).await {
            Ok(session_type) => match session_type {
                SessionType::RegisteredSession => true,
                SessionType::AnonymousSession => false,
            },
            Err(e) => {
                // If we can't determine session type, treat as anonymous/unregistered for safety
                ic_cdk::println!("Failed to get session type: {}", e);
                false
            }
        };

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

/// Get the current rate limit status for a principal on a specific property
#[query]
pub fn get_rate_limit_status(
    principal: Principal,
    property: String,
    is_registered: bool,
) -> Option<RateLimitStatus> {
    CANISTER_DATA.with(|data| {
        let data = data.borrow();
        data.get_rate_limit_status_with_property(&principal, &property, is_registered)
    })
}
