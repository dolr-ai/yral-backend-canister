pub mod canister_lifecycle;

use candid::{CandidType, Principal};
use ic_cdk::caller;
use ic_cdk_macros::{export_candid, query, update};
use serde::{Deserialize, Serialize};
use shared_utils::canister_specific::notification_store::types::error::NotificationStoreError;
use shared_utils::canister_specific::notification_store::types::notification::{
    Notification, NotificationData, NotificationType,
};
use shared_utils::canister_specific::notification_store::types::args::NotificationStoreInitArgs;
use shared_utils::common::utils::system_time::get_current_system_time_from_ic;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(CandidType, Serialize, Deserialize, Default)]
struct CanisterData {
    notifications: BTreeMap<Principal, Notification>,
    version: String,
}

mod memory {
    use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
    use ic_stable_structures::DefaultMemoryImpl;
    use std::cell::RefCell;

    type Memory = VirtualMemory<DefaultMemoryImpl>;

    thread_local! {
        static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
            RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    }

    const UPGRADES: MemoryId = MemoryId::new(0);

    pub fn get_upgrades_memory() -> Memory {
        MEMORY_MANAGER.with(|m| m.borrow().get(UPGRADES))
    }
}

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::new(CanisterData::default());
    static NEXT_ID: RefCell<u64> = RefCell::new(0);
}

#[update]
fn add_notification(notification_type: NotificationType) -> Result<(), NotificationStoreError> {
    let caller = caller();
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let next_id = NEXT_ID.with(|id| {
            let result = *id.borrow();
            *id.borrow_mut() += 1;
            result
        });
        let notifications = canister_data.notifications.entry(caller).or_default();
        notifications.0.push(NotificationData {
            notification_id: next_id,
            payload: notification_type,
            read: false,
            created_at: get_current_system_time_from_ic(),
        });
    });

    Ok(())
}

#[update]
fn mark_notification_as_read(notification_id: u64) -> Result<(), NotificationStoreError> {
    let caller = caller();
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        if let Some(notifications) = canister_data.notifications.get_mut(&caller) {
            if let Some(notification) = notifications
                .0
                .iter_mut()
                .find(|n| n.notification_id == notification_id)
            {
                notification.read = true;
            }
        }
    });

    Ok(())
}

#[query]
fn get_notifications(limit: usize, offset: usize) -> Vec<NotificationData> {
    let caller = caller();
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .notifications
            .get(&caller)
            .map(|notifications| {
                notifications
                    .0
                    .iter()
                    .skip(offset)
                    .take(limit)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    })
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

