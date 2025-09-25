use candid::Principal;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query};

use std::cell::RefCell;

mod api;
mod data_model;
mod util;

#[cfg(test)]
mod tests;

use data_model::{CanisterData, MissionProgress, MissionUpdateResult, UserDailyMissions};
use shared_utils::service::{update_version_from_args, GetVersion, SetVersion};
use std::borrow::Cow;

thread_local! {
    pub static CANISTER_DATA: RefCell<CanisterData> = RefCell::new(CanisterData::default());
}

// Re-export functions from API module so they're available as canister methods
pub use api::*;

#[derive(candid::CandidType, serde::Deserialize)]
pub struct DailyMissionsInitArgs {
    pub version: String,
    pub known_principal_ids:
        Option<shared_utils::common::types::known_principal::KnownPrincipalMap>,
}

impl From<DailyMissionsInitArgs> for shared_utils::service::ServiceInitArgs {
    fn from(args: DailyMissionsInitArgs) -> Self {
        shared_utils::service::ServiceInitArgs {
            version: args.version,
        }
    }
}

impl GetVersion for DailyMissionsInitArgs {
    fn get_version(&self) -> Cow<str> {
        self.version.as_str().into()
    }
}

#[init]
fn init(args: DailyMissionsInitArgs) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.set_version(&args.version);
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    use crate::data_model::memory;
    use ciborium::ser;
    use ic_stable_structures::writer::Writer;

    let mut state_bytes = vec![];
    CANISTER_DATA
        .with(|canister_data_ref_cell| {
            ser::into_writer(&*canister_data_ref_cell.borrow(), &mut state_bytes)
        })
        .expect("failed to encode state");

    let len = state_bytes.len() as u32;

    let mut upgrade_memory = memory::get_upgrades_memory();
    let mut writer = Writer::new(&mut upgrade_memory, 0);
    writer.write(&len.to_le_bytes()).unwrap();
    writer.write(&state_bytes).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    use crate::data_model::memory;
    use ciborium::de;
    use ic_stable_structures::reader::Reader;

    let heap_data = memory::get_upgrades_memory();
    let mut upgrade_reader = Reader::new(&heap_data, 0);

    let mut heap_data_len_bytes = [0; 4];
    upgrade_reader.read(&mut heap_data_len_bytes).unwrap();
    let heap_data_len = u32::from_le_bytes(heap_data_len_bytes) as usize;

    let mut canister_data_bytes = vec![0; heap_data_len];
    upgrade_reader.read(&mut canister_data_bytes).unwrap();

    let canister_data: CanisterData =
        de::from_reader(&*canister_data_bytes).expect("Failed to deserialize heap data");

    CANISTER_DATA.with_borrow_mut(|cdata| {
        *cdata = canister_data;
    });

    // Update version if provided in upgrade args
    update_version_from_args::<DailyMissionsInitArgs>(&CANISTER_DATA);
}

#[query]
fn get_version() -> String {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.version.clone())
}

// Export the candid interface
candid::export_service!();
