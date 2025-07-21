use ic_cdk::{init, post_upgrade, pre_upgrade};
use ic_cdk_macros::query;
use shared_utils::service::{ServiceInitArgs, StableState, update_version_from_args};

use crate::{CANISTER_DATA, data_model::memory};

#[query]
fn get_version() -> String {
    CANISTER_DATA.with(|data| data.borrow().version.clone())
}

#[init]
fn init(args: ServiceInitArgs) {
    CANISTER_DATA.with(|data| {
        data.borrow_mut().version = args.version;
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

    update_version_from_args::<ServiceInitArgs>(&CANISTER_DATA);
}