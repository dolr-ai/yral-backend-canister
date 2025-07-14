use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::{canister_specific::notification_store::types::{error::NotificationStoreError, notification::{Notification, NotificationData, NotificationType}}, common::utils::system_time::get_current_system_time_from_ic};

use crate::{CANISTER_DATA};

#[update]
fn add_notification(notification_type: NotificationType) -> Result<(), NotificationStoreError> {
    let caller = caller();
    CANISTER_DATA.with(|map| {
        let mut notifications = map.borrow().notifications.get(&caller).unwrap_or_default();
        let next_id = notifications.0.len() as u64;
        notifications.0.push(NotificationData { notification_id: next_id, payload: notification_type, read: false, created_at: get_current_system_time_from_ic()});
        map.borrow_mut().notifications.insert(caller, Notification (notifications.0));
    });

    Ok(())
}
