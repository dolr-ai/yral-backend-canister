use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::system_time::get_current_system_time_from_ic;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;
use std::time::Duration;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
pub fn prune_notifications() {
    CANISTER_DATA.with_borrow_mut(|map| {
        let now = get_current_system_time_from_ic();
        // Keep notifications from the last 30 days
           let thirty_days = Duration::from_secs(30 * 24 * 60 * 60);
        let thirty_days_ago = now
            .checked_sub(thirty_days)
            .unwrap_or(std::time::UNIX_EPOCH);

        let mut notifications: Vec<(Principal, Notification)> = map.notifications.iter().collect();

        notifications
            .iter_mut()
            .for_each(|(principal, notification)| {
                notification
                    .0
                    .retain(|data| data.created_at >= thirty_days_ago);

                map.notifications.insert(*principal, notification.clone());
            });
    });
}