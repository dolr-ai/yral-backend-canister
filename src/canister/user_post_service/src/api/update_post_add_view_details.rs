use ic_cdk_macros::update;
use shared_utils::canister_specific::user_post_service::types::{
    error::UserPostServiceError, storage::PostViewDetailsFromFrontend,
};

use crate::CANISTER_DATA;

#[update]
fn update_post_add_view_details(
    id: String,
    details: PostViewDetailsFromFrontend,
) -> Result<(), UserPostServiceError> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut post_to_update = canister_data.get_post(&id)?;

        post_to_update.add_view_details(&details);

        canister_data.add_post(post_to_update);
        Ok(())
    })
}
