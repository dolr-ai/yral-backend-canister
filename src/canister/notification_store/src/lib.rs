pub mod canister_lifecycle;
pub mod data_model;
pub mod api;

use candid::Principal;
use ic_cdk_macros::export_candid;
use shared_utils::canister_specific::notification_store::types::error::NotificationStoreError;
use shared_utils::canister_specific::notification_store::types::notification::{
    NotificationData, NotificationType,
};
use shared_utils::canister_specific::notification_store::types::args::NotificationStoreInitArgs;
use std::cell::RefCell;
use std::time::SystemTime;
use crate::data_model::CanisterData;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();

