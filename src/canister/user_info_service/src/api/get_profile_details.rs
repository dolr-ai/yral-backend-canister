use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::profile::{
    UserProfileDetailsForFrontendV3, UserProfileDetailsForFrontendV5,
    UserProfileDetailsForFrontendV6, UserProfileDetailsForFrontendV7,
};

use crate::CANISTER_DATA;

#[query]
pub fn get_user_profile_details(
    user: Principal,
) -> Result<UserProfileDetailsForFrontendV3, String> {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.get_profile_details_for_user(user))
}

#[query]
pub fn get_user_profile_details_v5(
    user: Principal,
) -> Result<UserProfileDetailsForFrontendV5, String> {
    CANISTER_DATA
        .with_borrow(|canister_data| canister_data.get_profile_details_for_user_v5(user, caller()))
}

#[query]
pub fn get_user_profile_details_v6(
    user: Principal,
) -> Result<UserProfileDetailsForFrontendV6, String> {
    CANISTER_DATA
        .with_borrow(|canister_data| canister_data.get_profile_details_for_user_v6(user, caller()))
}

#[query]
pub fn get_user_profile_details_v7(
    user: Principal,
) -> Result<UserProfileDetailsForFrontendV7, String> {
    CANISTER_DATA
        .with_borrow(|canister_data| canister_data.get_profile_details_for_user_v7(user, caller()))
}

#[query]
pub fn get_users_profile_details(
    users: Vec<Principal>,
) -> Result<Vec<UserProfileDetailsForFrontendV7>, String> {
    CANISTER_DATA
        .with_borrow(|canister_data| canister_data.get_users_profile_details(users, caller()))
}
