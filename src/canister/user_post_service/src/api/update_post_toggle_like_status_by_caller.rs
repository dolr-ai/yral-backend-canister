use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::canister_specific::user_post_service::types::error::UserPostServiceError;

use crate::CANISTER_DATA;

#[update]
pub fn update_post_toggle_like_status_by_caller(
    post_id: String,
) -> Result<bool, UserPostServiceError> {
    let caller_principal = caller();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut post = canister_data.get_post(&post_id)?;
        let response = post.toggle_like_status(&caller_principal);
        canister_data.add_post(post);
        Ok(response)
    })
}
