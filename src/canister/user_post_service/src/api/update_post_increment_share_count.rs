use ic_cdk_macros::update;
use shared_utils::canister_specific::user_post_service::types::error::UserPostServiceError;

use crate::CANISTER_DATA;

#[update]
fn update_post_increment_share_count(post_id: String) -> Result<u64, UserPostServiceError> {
    CANISTER_DATA.with_borrow_mut(|canister_data_ref_cell| {
        let mut post_to_update = canister_data_ref_cell.get_post(&post_id)?;

        let updated_share_count = post_to_update.increment_share_count();

        canister_data_ref_cell.add_post(post_to_update);

        Ok(updated_share_count)
    })
}
