use ciborium::de;
use ic_cdk_macros::post_upgrade;
use ic_stable_structures::Memory;

use crate::{CANISTER_DATA, data_model::memory::get_upgrades_memory};

#[post_upgrade]
fn post_upgrade() {
    restore_data_from_stable_memory();
}

fn restore_data_from_stable_memory() {
    let upgrades_memory = get_upgrades_memory();
    let mut state_len = [0; 4];

    upgrades_memory.read(0, &mut state_len);

    let state_len = u32::from_le_bytes(state_len) as usize;

    let mut state_bytes = vec![0; state_len];

    upgrades_memory.read(4, &mut state_bytes);

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        *canister_data = de::from_reader(&*state_bytes)
            .expect("Failed to deserialize canister data after upgrade");
    })
}
