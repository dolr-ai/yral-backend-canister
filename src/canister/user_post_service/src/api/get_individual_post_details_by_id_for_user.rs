use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::canister_specific::user_post_service::types::{
    args::PostDetailsForFrontend, error::UserPostServiceError,
};

use crate::CANISTER_DATA;

#[query]
fn get_individual_post_details_by_id_for_user(
    post_id: String,
    user: Principal,
) -> Result<PostDetailsForFrontend, UserPostServiceError> {
    CANISTER_DATA
        .with_borrow(|canister_data| canister_data.get_post_details_for_user(&post_id, user))
}
