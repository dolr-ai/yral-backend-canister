use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::{canister_specific::notification_store::types::{error::NotificationStoreError, notification::{NotificationData, NotificationType}}, common::utils::{permissions::is_caller_controller_or_global_admin, system_time::get_current_system_time_from_ic}};

use crate::{CANISTER_DATA};

#[update]
fn add_notification(user_principal: Principal, notification_type: NotificationType) -> Result<(), NotificationStoreError> {
    if user_principal != caller() && is_caller_controller_or_global_admin().is_err() {
        return Err(NotificationStoreError::Unauthorized);
    }

    CANISTER_DATA.with(|map| {
        let mut notification_data = map.borrow().notifications.get(&user_principal).unwrap_or_default();
        let next_id = notification_data.notifications.len() as u64;
        notification_data.notifications.push(NotificationData { 
            notification_id: next_id, 
            payload: notification_type, 
            created_at: get_current_system_time_from_ic()
        });
        
        if notification_data.notifications.len() >= 1000 {
            notification_data.notifications.drain(0..500);
            
            for (index, notification) in notification_data.notifications.iter_mut().enumerate() {
                notification.notification_id = index as u64;
            }
        }
        
        map.borrow_mut().notifications.insert(user_principal, notification_data);
    });

    Ok(())
}
