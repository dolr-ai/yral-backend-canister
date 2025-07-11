use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller")]
async fn update_profile_owner_for_individual_canisters() {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.user_principal_id_to_canister_id_map.iter().for_each(|(user_principal, user_canister_id)| {
            let _ = ic_cdk::notify(*user_canister_id, "update_profile_owner", (Some(*user_principal),));
        })
    });
}