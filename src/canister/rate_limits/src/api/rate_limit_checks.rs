use candid::Principal;
use ic_cdk_macros::{query, update};
use shared_utils::canister_specific::individual_user_template::types::session::SessionType;

use crate::{utils, CANISTER_DATA, RateLimitResult, RateLimitStatus};

/// Check if a principal has exceeded rate limits
#[update]
pub async fn check_rate_limit(principal: Principal) -> RateLimitResult {
    // Get session type from user_info_service
    let is_registered = match utils::get_session_type_for_principal(principal).await {
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
        if data.is_rate_limited(&principal, is_registered) {
            RateLimitResult::Err("Rate limit exceeded".to_string())
        } else {
            RateLimitResult::Ok("Within rate limit".to_string())
        }
    })
}

/// Increment the request count for a principal
#[update]
pub async fn increment_request_count(principal: Principal) -> RateLimitResult {
    // Get session type from user_info_service
    let is_registered = match utils::get_session_type_for_principal(principal).await {
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
        if data.is_rate_limited(&principal, is_registered) {
            RateLimitResult::Err("Rate limit exceeded".to_string())
        } else {
            data.increment_request(&principal);
            RateLimitResult::Ok("Request count incremented".to_string())
        }
    })
}

/// Get the current rate limit status for a principal
#[query]
pub fn get_rate_limit_status(principal: Principal) -> Option<RateLimitStatus> {
    CANISTER_DATA.with(|data| {
        let data = data.borrow();
        data.get_rate_limit_status(&principal)
    })
}