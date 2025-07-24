use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::canister_specific::user_post_service::types::storage::Post;

use crate::CANISTER_DATA;

#[query]
fn get_posts_of_this_user_profile_with_pagination_cursor(
    creator: Principal,
    limit: usize,
    offset: usize,
) -> Vec<Post> {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.get_posts_of_this_user_profile_with_pagination_cursor(creator, limit, offset)
    })
}
