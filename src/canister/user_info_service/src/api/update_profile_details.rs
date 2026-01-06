use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::user_info_service::types::{
        NSFWInfo, ProfileUpdateDetails, ProfileUpdateDetailsV2,
    },
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use crate::CANISTER_DATA;

#[update]
fn update_profile_details(details: ProfileUpdateDetails) -> Result<(), String> {
    CANISTER_DATA
        .with_borrow_mut(|canister_data| canister_data.update_profile_details(caller(), details))
}

#[update]
fn update_profile_details_v2(details: ProfileUpdateDetailsV2) -> Result<(), String> {
    CANISTER_DATA
        .with_borrow_mut(|canister_data| canister_data.update_profile_details_v2(caller(), details))
}

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_profile_nsfw_info(user_id: Principal, nsfw_info: NSFWInfo) -> Result<(), String> {
    CANISTER_DATA
        .with_borrow_mut(|canister_data| canister_data.update_profile_nsfw_info(user_id, nsfw_info))
}

/// Admin-only endpoint to update the AI influencer status for a user's profile
#[update(guard = "is_caller_controller_or_global_admin")]
fn update_profile_ai_influencer_status(
    user_id: Principal,
    is_ai_influencer: bool,
) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.update_profile_ai_influencer_status(user_id, is_ai_influencer)
    })
}
