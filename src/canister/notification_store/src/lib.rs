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

const PRUNING_INTERVAL_IN_NANOS: u64 = 30 * 24 * 60 * 60 * 1_000_000_000;

fn set_pruning_timer() {
    ic_cdk_timers::set_timer_interval(Duration::from_nanos(PRUNING_INTERVAL_IN_NANOS), move || {
        CANISTER_DATA.with_borrow_mut(|map| {
            let now = get_current_system_time_from_ic();

            // Collecting the user principals first to avoid borrowing issues while mutating the map
            let users: Vec<Principal> = map.notifications.iter().map(|(user, _)| user).collect();

            for user in users {
                if let Some(mut notifications) = map.notifications.get(&user) {
                    notifications.0.retain(|n| n.created_at > now);

                    map.notifications.insert(user, notifications);
                }
            }
        })
    });
}

export_candid!();

