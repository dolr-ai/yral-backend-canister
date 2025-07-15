pub mod memory;
use ic_stable_structures::{memory_manager::{VirtualMemory}, DefaultMemoryImpl, StableBTreeMap};

use candid::Principal;
use serde::{Deserialize, Serialize};
use shared_utils::canister_specific::notification_store::types::notification::Notification;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    #[serde(skip, default = "_init_notifications")]
    pub notifications: StableBTreeMap<Principal, Notification, Memory>,
    pub version: String,
}

impl Default for CanisterData{
    fn default() -> Self {
        Self { notifications: _init_notifications(), version: String::from("v1.0.0") }
    }
}

fn _init_notifications() -> StableBTreeMap<Principal, Notification, Memory> {
    StableBTreeMap::init(memory::get_notification_memory())
}