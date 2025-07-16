use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::update;
use shared_utils::canister_specific::individual_user_template::types::post::{Post, PostDetailsFromFrontend};
use shared_utils::common::utils::system_time::get_current_system_time_from_ic;

use crate::{CANISTER_DATA};
use crate::types::error::UserPostServiceError;
use crate::types::storage::{PostWrapper, PostIdList};

#[update]
fn add_post(post_details: PostDetailsFromFrontend) -> Result<u64, UserPostServiceError> {
    let creator: Principal = caller();
    let current_time = get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let post_id = canister_data.next_post_id;
        canister_data.next_post_id += 1;

        let post = Post::new(post_id, &post_details, &current_time);
        canister_data.posts.insert(post_id, PostWrapper(post));

        let mut id_list = canister_data
            .posts_by_creator
            .get(&creator)
            .unwrap_or_default();

        let mut vec = id_list.0;
        vec.insert(0, post_id); // most recent at front
        canister_data.posts_by_creator.insert(creator, PostIdList(vec));

        Ok(post_id)
    })
} 