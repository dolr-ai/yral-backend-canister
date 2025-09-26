use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::canister_specific::user_info_service::types::ProfileUpdateDetails;

use crate::CANISTER_DATA;

#[update]
fn update_profile_details(details: ProfileUpdateDetails) -> Result<(), String> {
    CANISTER_DATA
        .with_borrow_mut(|canister_data| canister_data.update_profile_details(caller(), details))
}
