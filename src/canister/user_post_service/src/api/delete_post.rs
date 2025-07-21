use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::canister_specific::user_post_service::types::error::UserPostServiceError;

use crate::CANISTER_DATA;

#[update]
fn delete_post(post_id: String) -> Result<(), UserPostServiceError> {
    CANISTER_DATA.with_borrow_mut(|canister_data| canister_data.delete_post(&post_id, caller()))
}
