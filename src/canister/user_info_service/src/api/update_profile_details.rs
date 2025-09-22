use ic_cdk::caller;
use ic_cdk_macros::update;

use crate::CANISTER_DATA;
use crate::data_model::ProfileUpdateDetails;

#[update]
fn update_profile_details(details: ProfileUpdateDetails) -> Result<(), String> {
    CANISTER_DATA
        .with_borrow_mut(|canister_data| canister_data.update_profile_details(caller(), details))
}
