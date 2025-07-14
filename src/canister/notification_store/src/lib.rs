pub mod canister_lifecycle;
pub mod data_model;
pub mod api;

use ic_cdk_macros::export_candid;
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
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::new(CanisterData::default());
    static NEXT_ID: RefCell<u64> = RefCell::new(0);
}

const THIRTY_DAYS_IN_NANOS: u64 = 30 * 24 * 60 * 60 * 1_000_000_000;

fn set_pruning_timer() {
    ic_cdk_timers::set_timer_interval(Duration::from_secs(60 * 60 * 24 * 30), move || {
        CANISTER_DATA.with_borrow_mut(|canister_data| {
            let now_nanos = get_current_system_time_from_ic()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;

            for notifications in canister_data.notifications.values_mut() {
                notifications.0.retain(|n| {
                    let created_at_nanos = n
                        .created_at
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;
                    now_nanos.saturating_sub(created_at_nanos) < THIRTY_DAYS_IN_NANOS
                });
            }
        })
    });
}

export_candid!();

