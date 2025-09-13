use candid::Principal;
use std::cell::RefCell;

use ic_cdk::export_candid;
use shared_utils::canister_specific::individual_user_template::types::profile::{UserProfileDetailsForFrontendV3, UserProfileDetailsForFrontendV4};
use shared_utils::canister_specific::individual_user_template::types::session::SessionType;
use shared_utils::canister_specific::user_info_service::args::UserInfoServiceInitArgs;

use crate::data_model::{CanisterData, ProfileUpdateDetails};
use crate::api::get_followers::FollowersResponse;
use crate::api::get_following::FollowingResponse;

mod api;
pub mod data_model;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();
