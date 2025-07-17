use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;

use crate::{CANISTER_DATA};
use shared_utils::canister_specific::user_post_service::types::error::UserPostServiceError;
use shared_utils::canister_specific::user_post_service::types::storage::{Post, PostIdList};

#[update]
fn add_post(post: Post) -> Result<u64, UserPostServiceError> {
    let creator: Principal = caller();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let post_id = canister_data.next_post_id;
        canister_data.next_post_id += 1;

        canister_data.posts.insert(post_id, post);

        let id_list = canister_data
            .posts_by_creator
            .get(&creator)
            .unwrap_or_default();

        let mut vec = id_list.0;
        vec.push(post_id);
        canister_data.posts_by_creator.insert(creator, PostIdList(vec));

        Ok(post_id)
    })
} 