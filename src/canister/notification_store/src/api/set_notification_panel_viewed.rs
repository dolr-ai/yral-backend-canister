use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::{canister_specific::notification_store::types::error::NotificationStoreError, common::utils::system_time::get_current_system_time_from_ic};

use crate::CANISTER_DATA;

#[update]
fn set_notification_panel_viewed() -> Result<(), NotificationStoreError> {
    let caller_principal = caller();
    let current_time = get_current_system_time_from_ic();
    
    CANISTER_DATA.with(|data| {
        let mut data_mut = data.borrow_mut();
        if let Some(mut notification_data) = data_mut.notifications.get(&caller_principal) {
            notification_data.last_viewed = Some(current_time);
            data_mut.notifications.insert(caller_principal, notification_data);
        } else {
            let notification_data = shared_utils::canister_specific::notification_store::types::notification::Notification {
                notifications: vec![],
                last_viewed: Some(current_time),
            };
            data_mut.notifications.insert(caller_principal, notification_data);
        }
    });
    
    Ok(())
}