use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
fn register_new_user(user_principal: Principal) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| canister_data.register_new_user(user_principal))
}
