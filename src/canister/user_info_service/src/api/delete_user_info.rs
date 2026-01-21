use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_not_anonymous;

use crate::CANISTER_DATA;

#[update(guard = "is_not_anonymous")]
fn delete_user_info(principal_to_delete: Principal) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.delete_user_info(principal_to_delete, caller())
    })
}
