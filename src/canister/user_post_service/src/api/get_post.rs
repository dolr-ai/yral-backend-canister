use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::post::Post;

use crate::CANISTER_DATA;
use crate::types::storage::PostWrapper;

#[query]
fn get_post(post_id: u64) -> Option<Post> {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .posts
            .get(&post_id)
            .map(|wrapper| wrapper.0.clone())
    })
} 