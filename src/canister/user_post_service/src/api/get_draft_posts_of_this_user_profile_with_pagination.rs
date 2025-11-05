use ic_cdk::caller;
use ic_cdk_macros::query;
use shared_utils::{
    canister_specific::{
        individual_user_template::types::error::GetPostsOfUserProfileError,
        user_post_service::types::storage::Post,
    },
    common::utils::permissions::is_not_anonymous,
};

use crate::CANISTER_DATA;

#[query(guard = "is_not_anonymous")]
fn get_draft_posts_of_this_user_profile_with_pagination(
    offset: usize,
    limit: usize,
) -> Result<Vec<Post>, GetPostsOfUserProfileError> {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.get_draft_posts_of_this_user_profile_with_pagination(caller(), limit, offset)
    })
}
