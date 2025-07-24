use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontendV3;

use crate::CANISTER_DATA;

#[query]
pub fn get_user_profile_details(
    user: Principal,
) -> Result<UserProfileDetailsForFrontendV3, String> {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.get_profile_details_for_user(user))
}
