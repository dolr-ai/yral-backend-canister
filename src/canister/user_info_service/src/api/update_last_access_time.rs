use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_last_access_time() -> Result<(), String> {
    CANISTER_DATA
        .with_borrow_mut(|canister_data| canister_data.update_last_access_time_for_user(caller()))
}
