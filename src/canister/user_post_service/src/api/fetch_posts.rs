use ic_cdk_macros::query;

use shared_utils::canister_specific::user_post_service::types::args::{
    FetchPostsArgs, FetchPostsResult,
};

use crate::CANISTER_DATA;

#[query]
fn fetch_posts(fetch_posts_args: FetchPostsArgs) -> FetchPostsResult {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.fetch_posts(fetch_posts_args.last_uuid_processed, fetch_posts_args.limit)
    })
}
