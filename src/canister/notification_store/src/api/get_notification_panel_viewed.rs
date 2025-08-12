use std::time::SystemTime;

use ic_cdk::caller;
use ic_cdk_macros::query;

use crate::CANISTER_DATA;

#[query]
fn get_notification_panel_viewed() -> Option<SystemTime> {
    let caller_principal = caller();
    
    CANISTER_DATA.with(|data| {
        let data_ref = data.borrow();
        data_ref.notifications.get(&caller_principal)
            .and_then(|notification_data| notification_data.last_viewed)
    })
}