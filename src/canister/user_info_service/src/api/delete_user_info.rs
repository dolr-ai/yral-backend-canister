use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::{query, update};
use shared_utils::common::utils::permissions::{is_caller_global_admin, is_not_anonymous};

use crate::CANISTER_DATA;

#[update(guard = "is_not_anonymous")]
fn delete_user_info(user_principal: Principal) -> Result<(), String> {
    if is_caller_global_admin().is_ok() || caller() == user_principal {
        CANISTER_DATA
            .with_borrow_mut(|canister_data| canister_data.delete_user_info(user_principal))
    } else {
        Err("Unauthorized".into())
    }
}

#[update(guard = "is_not_anonymous")]
fn delete_bot_account(bot_principal: Principal) -> Result<(), String> {
    CANISTER_DATA
        .with_borrow_mut(|canister_data| canister_data.delete_bot_account(bot_principal, caller()))
}
