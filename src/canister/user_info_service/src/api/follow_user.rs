use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_not_anonymous;

use crate::CANISTER_DATA;

#[update(guard = "is_not_anonymous")]
fn follow_user(target: Principal) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.follow_user(caller(), target)
    })
}