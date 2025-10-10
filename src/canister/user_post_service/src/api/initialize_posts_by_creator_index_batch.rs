use shared_utils::common::utils::permissions::is_caller_controller;

use crate::CANISTER_DATA;

#[ic_cdk_macros::update(guard = "is_caller_controller")]
fn initialize_posts_by_creator_index_batch(
    last_uuid_processed: Option<String>,
    limit: usize,
) -> Option<String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.initialize_posts_by_creator_index(last_uuid_processed, limit)
    })
}
