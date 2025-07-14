pub mod memory;

use std::collections::BTreeMap;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use shared_utils::canister_specific::notification_store::types::notification::Notification;

#[derive(CandidType, Serialize, Deserialize, Default)]
pub struct CanisterData {
    pub notifications: BTreeMap<Principal, Notification>,
    pub version: String,
}