use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;

use crate::CANISTER_DATA;

#[update]
fn unfollow_user(target: Principal) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| canister_data.unfollow_user(caller(), target))
}
