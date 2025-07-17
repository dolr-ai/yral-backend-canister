use ic_cdk_macros::query;
use shared_utils::canister_specific::user_post_service::types::storage::Post;

use crate::CANISTER_DATA;

#[query]
fn get_post(post_id: u64) -> Option<Post> {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .posts
            .get(&post_id)
    })
} 