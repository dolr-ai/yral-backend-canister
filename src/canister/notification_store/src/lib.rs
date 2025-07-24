pub mod canister_lifecycle;
pub mod data_model;
pub mod api;

use candid::Principal;
use ic_cdk_macros::export_candid;
use ic_stable_structures::memory_manager::MemoryManager;
use ic_stable_structures::DefaultMemoryImpl;
use shared_utils::canister_specific::notification_store::types::error::NotificationStoreError;
use shared_utils::canister_specific::notification_store::types::notification::{
    NotificationData, NotificationType,
};
use shared_utils::canister_specific::notification_store::types::args::NotificationStoreInitArgs;
use shared_utils::common::utils::system_time::get_current_system_time_from_ic;
use std::cell::RefCell;
use std::time::Duration;

use crate::data_model::CanisterData;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();

