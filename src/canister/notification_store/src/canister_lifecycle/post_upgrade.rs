use crate::{memory, set_pruning_timer, CANISTER_DATA};
use ciborium::de;
use ic_cdk_macros::post_upgrade;
use ic_stable_structures::Memory;
use shared_utils::canister_specific::notification_store::types::args::NotificationStoreInitArgs;

#[post_upgrade]
pub fn post_upgrade() {
    restore_data_from_stable_memory();
    update_version_from_args();
    set_pruning_timer();
}

fn restore_data_from_stable_memory() {
    let upgrades_memory = memory::get_upgrades_memory();
    let mut state_len = [0; 4];

    upgrades_memory.read(0, &mut state_len);

    let state_len = u32::from_le_bytes(state_len) as usize;

    let mut state_bytes = vec![0; state_len];

    upgrades_memory.read(4, &mut state_bytes);

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        *canister_data =
            de::from_reader(&*state_bytes).expect("Failed to deserialize canister data after upgrade");
    })
}

fn update_version_from_args() {
    let raw_args = ic_cdk::api::call::arg_data_raw();
    let (upgrade_args,): (NotificationStoreInitArgs,) =
        candid::decode_one(&raw_args).expect("Failed to decode upgrade args");

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.version = upgrade_args.version;
    });
}
