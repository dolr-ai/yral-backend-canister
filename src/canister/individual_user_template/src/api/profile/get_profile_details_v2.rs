use crate::CANISTER_DATA;
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontendV2;

#[query]
fn get_profile_details_v2() -> UserProfileDetailsForFrontendV2 {
    CANISTER_DATA.with_borrow(|canister_data_ref_cell| {
        let profile = canister_data_ref_cell.profile.clone();

        UserProfileDetailsForFrontendV2 {
            principal_id: profile.principal_id.unwrap(),
            display_name: None,
            unique_user_name: None,
            profile_picture_url: profile.profile_picture.as_ref().map(|p| p.url.clone()),
            profile_stats: profile.profile_stats,
            followers_count: 0,
            following_count: 0,
            lifetime_earnings: 0,
            referrer_details: profile.referrer_details,
            migration_info: canister_data_ref_cell.migration_info,
        }
    })
}
