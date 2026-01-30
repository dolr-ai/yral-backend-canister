use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_not_anonymous;

use crate::CANISTER_DATA;

#[update(guard = "is_not_anonymous")]
fn register_new_user() -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| canister_data.register_new_user(caller()))
}

#[update(guard = "is_not_anonymous")]
fn register_new_user_v2(bot_account_principal: Option<Principal>) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.register_new_user_v2(bot_account_principal)
    })
}
