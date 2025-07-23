use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::system_time::get_current_system_time_from_ic;
use shared_utils::common::utils::permissions::is_caller_controller_or_global_admin;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
pub fn prune_notifications() {
    CANISTER_DATA.with_borrow_mut(|map| {
        let now = get_current_system_time_from_ic();

        // Collecting the user principals first to avoid borrowing issues while mutating the map
        let users: Vec<Principal> = map.notifications.iter().map(|(user, _)| user).collect();

        for user in users {
            if let Some(mut notifications) = map.notifications.get(&user) {
                notifications.0.retain(|n| n.created_at > now);

                map.notifications.insert(user, notifications);
            }
        }
    })
}