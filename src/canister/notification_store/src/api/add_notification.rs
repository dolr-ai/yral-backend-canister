use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::{canister_specific::notification_store::types::{error::NotificationStoreError, notification::{Notification, NotificationData, NotificationType}}, common::utils::{permissions::is_caller_controller, system_time::get_current_system_time_from_ic}};

use crate::{CANISTER_DATA};

#[update]
fn add_notification(user_principal: Principal, notification_type: NotificationType) -> Result<(), NotificationStoreError> {
    if user_principal != caller() || is_caller_controller().is_err() {
        return Err(NotificationStoreError::Unauthorized);
    }

    CANISTER_DATA.with(|map| {
        let mut notifications = map.borrow().notifications.get(&user_principal).unwrap_or_default();
        let next_id = notifications.0.len() as u64;
        notifications.0.push(NotificationData { notification_id: next_id, payload: notification_type, read: false, created_at: get_current_system_time_from_ic()});
        map.borrow_mut().notifications.insert(user_principal, Notification (notifications.0));
    });

    Ok(())
}
