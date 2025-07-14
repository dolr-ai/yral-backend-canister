use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::export_candid;
use shared_utils::canister_specific::notification_store::types::error::NotificationStoreError;
use shared_utils::canister_specific::notification_store::types::notification::{Notification, NotificationData, NotificationType};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use shared_utils::common::utils::system_time::get_current_system_time_from_ic;
use std::cell::RefCell;
use std::time::Duration;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static NEXT_ID: RefCell<u64> = RefCell::new(0);

    static CANISTER_DATA: RefCell<StableBTreeMap<Principal, Notification, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}


#[ic_cdk_macros::update]
fn add_notification(notification_type: NotificationType) -> Result<(), NotificationStoreError> {
    let caller = caller();
    CANISTER_DATA.with(|map| {
        let next_id = NEXT_ID.with(|id| *id.borrow_mut());
        let mut notifications = map.borrow().get(&caller).unwrap_or_default();
        notifications.0.push(NotificationData { notification_id: next_id, payload: notification_type, read: false, created_at: get_current_system_time_from_ic()});
        map.borrow_mut().insert(caller, Notification (notifications.0));
        NEXT_ID.with(|id| *id.borrow_mut() += 1);
    });

    Ok(())
}

#[ic_cdk_macros::update]
fn mark_notification_as_read(notification_id: u64) -> Result<(), NotificationStoreError>{
    let caller = caller();
    CANISTER_DATA.with_borrow_mut(|map| {
        let mut notifications = map.get(&caller).unwrap();

        notifications.0.iter_mut().find(|n| n.notification_id == notification_id).unwrap().read = true;

        map.insert(caller, notifications);
    });

    Ok(())
}

#[ic_cdk_macros::query]
fn get_notifications(limit: usize, offset: usize) -> Vec<NotificationData> {
    let caller = caller();
    CANISTER_DATA.with(|map| {
        let notifications = map.borrow().get(&caller).unwrap_or_default();
        notifications.0.iter().skip(offset).take(limit).cloned().collect()
    })
}

#[ic_cdk_macros::init]
fn init() {

    // pruning notifications older than 30 days
    ic_cdk_timers::set_timer_interval(Duration::from_secs(60 * 60 * 24 * 30), move || {
        CANISTER_DATA.with_borrow_mut(|map|{
            let now = get_current_system_time_from_ic();

            // Collecting the user principals first to avoid borrowing issues while mutating the map
            let users: Vec<Principal> = map
                .iter()
                .map(|(user, _)| user)
                .collect();

            for user in users {
                if let Some(mut notifications) = map.get(&user) {
                    notifications
                        .0
                        .retain(|n| n.created_at > now);

                    map.insert(user, notifications);
                }
            }
        })
    });
}
#[ic_cdk_macros::post_upgrade]
pub fn post_upgrade(){
        // pruning notifications older than 30 days
        ic_cdk_timers::set_timer_interval(Duration::from_secs(60 * 60 * 24 * 30), move || {
            CANISTER_DATA.with_borrow_mut(|map|{
                let now = get_current_system_time_from_ic();
    
                // Collecting the user principals first to avoid borrowing issues while mutating the map
                let users: Vec<Principal> = map
                    .iter()
                    .map(|(user, _)| user)
                    .collect();
    
                for user in users {
                    if let Some(mut notifications) = map.get(&user) {
                        notifications
                            .0
                            .retain(|n| n.created_at > now);
    
                        map.insert(user, notifications);
                    }
                }
            })
        });
}

export_candid!();

