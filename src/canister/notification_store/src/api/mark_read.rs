use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::{canister_specific::notification_store::types::error::NotificationStoreError};

use crate::CANISTER_DATA;

#[update]
fn mark_notification_as_read(notification_id: u64) -> Result<(), NotificationStoreError> {
    CANISTER_DATA.with_borrow_mut(|map| {
        let mut notifications = map.notifications.get(&caller()).unwrap();

        notifications
            .0
            .iter_mut()
            .find(|n| n.notification_id == notification_id)
            .unwrap()
            .read = true;

        map.notifications.insert(caller(), notifications);
    });

    Ok(())
}
