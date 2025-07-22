use ic_cdk::{init, post_upgrade, pre_upgrade};
use ic_cdk_macros::query;
use shared_utils::service::{ServiceInitArgs, StableState, update_version_from_args};

use crate::{DEDUP_INDEX, data_model::memory};

#[query]
fn get_version() -> String {
    DEDUP_INDEX.with_borrow(|canister_data| canister_data.version.clone())
}

#[init]
fn init(args: ServiceInitArgs) {
    DEDUP_INDEX.with_borrow_mut(|canister_data| {
        canister_data.version = args.version;
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    StableState::save(&DEDUP_INDEX, &mut memory::get_upgrades_memory()).expect("state to be saved");
}

#[post_upgrade]
pub fn post_upgrade() {
    StableState::restore(&DEDUP_INDEX, &mut memory::get_upgrades_memory())
        .expect("state to be restored");

    update_version_from_args::<ServiceInitArgs>(&DEDUP_INDEX);
}
