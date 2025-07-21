use ic_cdk::{init, post_upgrade, pre_upgrade};
use ic_cdk_macros::query;
use shared_utils::service::{StableState, update_version_from_args};

use crate::{CANISTER_DATA, data_model::memory, types::RateLimitsInitArgs};

#[query]
fn get_version() -> String {
    CANISTER_DATA.with(|data| data.borrow().version.clone())
}

#[init]
fn init(args: RateLimitsInitArgs) {
    CANISTER_DATA.with(|data| {
        let mut data = data.borrow_mut();
        data.version = args.version;
        data.user_info_canister = args.user_info_canister;
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    StableState::save(&CANISTER_DATA, &mut memory::get_upgrades_memory()).expect("state to be saved");
}

#[post_upgrade]
pub fn post_upgrade() {
    StableState::restore(&CANISTER_DATA, &mut memory::get_upgrades_memory())
        .expect("state to be restored");

    update_version_from_args::<RateLimitsInitArgs>(&CANISTER_DATA);
}