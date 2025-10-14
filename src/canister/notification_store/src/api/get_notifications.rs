use ic_cdk::caller;
use ic_cdk_macros::query;
use shared_utils::canister_specific::notification_store::types::notification::NotificationData;

use crate::CANISTER_DATA;

#[query]
fn get_notifications(limit: usize, offset: usize) -> Vec<NotificationData> {
    let caller = caller();
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .notifications
            .get(&caller)
            .map(|notification_data| {
                let total = notification_data.notifications.len();
                if offset >= total {
                    return Vec::new();
                }
                
                let end = total.saturating_sub(offset);
                let start = end.saturating_sub(limit);
                
                notification_data
                    .notifications[start..end]
                    .iter()
                    .rev()
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    })
}
