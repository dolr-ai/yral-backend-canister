use candid::Principal;
use ic_cdk::update;
use shared_utils::common::utils::permissions::{is_caller_global_admin, is_not_anonymous};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_global_admin")]
fn accept_new_user_registration(user_principal: Principal, authenticated: bool) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.register_authenticated_user(user_principal, authenticated)
    })
}

#[update(guard = "is_not_anonymous")]
fn accept_new_user_registration_v2(
    new_principal: Principal,
    authenticated: bool,
    main_account: Option<Principal>,
) -> Result<(), String> {
    if let Some(main_account_principal) = main_account {
        if ic_cdk::caller() != main_account_principal {
            return Err("Caller does not match main account".to_string());
        }
    }

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.register_authenticated_user_v2(new_principal, authenticated, main_account)
    })
}
