use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::user_info_service::types::SubscriptionPlan,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
fn change_subscription_plan(user_id: Principal, new_plan: SubscriptionPlan) -> Result<(), String> {
    CANISTER_DATA
        .with_borrow_mut(|canister_data| canister_data.change_subscription_plan(user_id, new_plan))
}

#[update(guard = "is_caller_controller_or_global_admin")]
fn remove_pro_plan_free_video_credits(
    user_id: Principal,
    credits_to_remove: u32,
) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.remove_pro_plan_free_video_credits(user_id, credits_to_remove)
    })
}

#[update(guard = "is_caller_controller_or_global_admin")]
fn add_pro_plan_free_video_credits(user_id: Principal, credits_to_add: u32) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.add_pro_plan_free_video_credits(user_id, credits_to_add)
    })
}
