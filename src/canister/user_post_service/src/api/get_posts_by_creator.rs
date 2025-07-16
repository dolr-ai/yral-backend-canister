use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::canister_specific::individual_user_template::types::post::Post;

use crate::CANISTER_DATA;
use crate::types::storage::{PostIdList};

#[query]
fn get_posts_by_creator(creator: Principal, limit: usize, offset: usize) -> Vec<Post> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let id_list = canister_data
            .posts_by_creator
            .get(&creator)
            .unwrap_or_default();

        id_list
            .0
            .iter()
            .skip(offset)
            .take(limit)
            .filter_map(|id| canister_data.posts.get(id).map(|w| w.0.clone()))
            .collect()
    })
} 