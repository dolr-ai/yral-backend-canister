use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::{canister_specific::notification_store::types::error::NotificationStoreError};

use crate::CANISTER_DATA;

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
