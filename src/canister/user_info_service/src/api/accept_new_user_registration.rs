use candid::Principal;
use ic_cdk::update;
use shared_utils::common::utils::permissions::is_caller_global_admin;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_global_admin")]
fn accept_new_user_registration(user_principal: Principal, authenticated: bool) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.register_authenticated_user(user_principal, authenticated)
    })
}

#[update(guard = "is_caller_global_admin")]
fn accept_new_user_registration_v2(
    user_principal: Principal,
    authenticated: bool,
    bot_principal: Option<Principal>,
) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.register_authenticated_user_v2(user_principal, authenticated, bot_principal)
    })
}
