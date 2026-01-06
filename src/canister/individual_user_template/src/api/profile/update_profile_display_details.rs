use crate::{util::cycles::notify_to_recharge_canister, CANISTER_DATA};
use candid::CandidType;
use ic_cdk_macros::update;
use shared_utils::canister_specific::{
    individual_user_template::types::profile::{
        UserProfileDetailsForFrontend, UserProfileUpdateDetailsFromFrontend,
    },
    user_info_service::types::ProfilePictureData,
};

#[derive(CandidType)]
pub enum UpdateProfileDetailsError {
    NotAuthorized,
}

/// # Access Control
/// Only the user whose profile details are stored in this canister can update their details.
#[update]
fn update_profile_display_details(
    user_profile_details: UserProfileUpdateDetailsFromFrontend,
) -> Result<UserProfileDetailsForFrontend, UpdateProfileDetailsError> {
    notify_to_recharge_canister();
    // * access control
    let current_caller = ic_cdk::caller();
    let my_principal_id = CANISTER_DATA
        .with(|canister_data_ref_cell| canister_data_ref_cell.borrow().profile.principal_id)
        .unwrap();

    if current_caller != my_principal_id {
        return Err(UpdateProfileDetailsError::NotAuthorized);
    }

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let profile = &mut canister_data_ref_cell.borrow_mut().profile;

        if let Some(url) = user_profile_details.profile_picture_url {
            // Keep existing nsfw_info or default to safe values
            let nsfw_info = profile
                .profile_picture
                .as_ref()
                .map(|p| p.nsfw_info.clone())
                .unwrap_or_default();
            profile.profile_picture = Some(ProfilePictureData { url, nsfw_info });
        }
    });

    Ok(CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.borrow();
        let profile = &canister_data.profile;

        UserProfileDetailsForFrontend {
            principal_id: profile.principal_id.unwrap(),
            display_name: None,
            unique_user_name: None,
            profile_picture_url: profile.profile_picture.as_ref().map(|p| p.url.clone()),
            profile_stats: profile.profile_stats,
            followers_count: 0,
            following_count: 0,
            lifetime_earnings: 0,
            referrer_details: profile.referrer_details.clone(),
        }
    }))
}
