use candid::Principal;
use ic_cdk_macros::{query, update};
use shared_utils::{
    canister_specific::rate_limits::{VideoGenRequest, VideoGenRequestKey, VideoGenRequestStatus},
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use crate::CANISTER_DATA;

/// Create a new video generation request after checking rate limits
#[update(guard = "is_caller_controller_or_global_admin")]
pub async fn create_video_generation_request(
    principal: Principal,
    model_name: String,
    prompt: String,
    property: String,
    is_registered: bool,
) -> Result<VideoGenRequestKey, String> {
    // Validate inputs
    if model_name.is_empty() || model_name.len() > 100 {
        return Err("Invalid model name".to_string());
    }
    if prompt.is_empty() || prompt.len() > 1000 {
        return Err("Invalid prompt length".to_string());
    }
    if property.is_empty() || property.len() > 50 {
        return Err("Invalid property".to_string());
    }

    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();

        // Check rate limits first
        if data.is_rate_limited_with_property(&principal, &property, is_registered) {
            return Err("Rate limit exceeded".to_string());
        }

        // Increment rate limit counter
        data.increment_request_with_property(&principal, &property);

        // Create the video generation request
        let key = data.create_video_gen_request(principal, model_name, prompt);
        Ok(key)
    })
}

/// Update video generation request status (admin/offchain agent only)
#[update(guard = "is_caller_controller_or_global_admin")]
pub async fn update_video_generation_status(
    key: VideoGenRequestKey,
    status: VideoGenRequestStatus,
) -> Result<(), String> {
    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.update_video_gen_request_status(&key, status)
    })
}

/// Get a specific video generation request
#[query]
pub fn get_video_generation_request(key: VideoGenRequestKey) -> Option<VideoGenRequest> {
    CANISTER_DATA.with(|data| {
        let data = data.borrow();
        data.get_video_gen_request(&key)
    })
}

/// Get recent video generation requests for a user with cursor-based pagination
/// - start: Optional starting counter (cursor). If not provided, starts from the most recent
/// - limit: Optional limit on number of results (default 10, max 100)
#[query]
pub fn get_user_video_generation_requests(
    principal: Principal,
    start: Option<u64>,
    limit: Option<u64>,
) -> Vec<(VideoGenRequestKey, VideoGenRequest)> {
    CANISTER_DATA.with(|data| {
        let data = data.borrow();
        data.get_user_video_gen_requests(principal, start, limit)
    })
}

/// Poll for video generation status - simplified endpoint for clients
#[query]
pub fn poll_video_generation_status(
    key: VideoGenRequestKey,
) -> Result<VideoGenRequestStatus, String> {
    CANISTER_DATA.with(|data| {
        let data = data.borrow();
        data.get_video_gen_request(&key)
            .map(|request| request.status)
            .ok_or_else(|| "Video generation request not found".to_string())
    })
}

/// Decrement the rate limit counter for a failed video generation request
#[update(guard = "is_caller_controller_or_global_admin")]
pub async fn decrement_video_generation_counter(
    key: VideoGenRequestKey,
    property: String,
) -> Result<(), String> {
    // Validate property
    if property.is_empty() || property.len() > 50 {
        return Err("Invalid property".to_string());
    }

    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        
        // Verify the request exists and is in failed state
        if let Some(request) = data.get_video_gen_request(&key) {
            match request.status {
                VideoGenRequestStatus::Failed(_) => {
                    // Decrement the counter for the principal and property
                    data.decrement_request_with_property(&key.principal, &property);
                    Ok(())
                }
                _ => Err("Can only decrement counter for failed requests".to_string())
            }
        } else {
            Err("Video generation request not found".to_string())
        }
    })
}
