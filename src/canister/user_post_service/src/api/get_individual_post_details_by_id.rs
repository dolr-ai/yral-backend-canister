use ic_cdk_macros::query;
use shared_utils::canister_specific::user_post_service::types::{
    error::UserPostServiceError, storage::Post,
};

use crate::CANISTER_DATA;

#[query]
pub fn get_individual_post_details_by_id(post_id: String) -> Result<Post, UserPostServiceError> {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.get_post(&post_id))
}
