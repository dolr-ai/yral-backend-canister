use candid::Principal;
use ic_cdk::update;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
fn convert_to_bot_account(bot_principal: Principal, owner: Principal) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.convert_to_bot_account(bot_principal, owner)
    })
}
