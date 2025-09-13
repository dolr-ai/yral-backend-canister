use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontendV4;

use crate::CANISTER_DATA;

#[query]
fn get_profile_details_v4(user_principal: Principal) -> Result<UserProfileDetailsForFrontendV4, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.get_profile_details_v4(user_principal)
    })
}