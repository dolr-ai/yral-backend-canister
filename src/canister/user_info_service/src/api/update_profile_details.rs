use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::common::utils::permissions::is_not_anonymous;

use crate::data_model::ProfileUpdateDetails;
use crate::CANISTER_DATA;

#[update(guard = "is_not_anonymous")]
fn update_profile_details(details: ProfileUpdateDetails) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.update_profile_details(caller(), details)
    })
}