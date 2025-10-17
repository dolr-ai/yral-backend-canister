use ic_cdk_macros::update;
use shared_utils::common::{
    types::top_posts::post_score_index_item::PostStatus, utils::permissions::is_caller_global_admin,
};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_global_admin")]
fn update_post_status(id: String, status: PostStatus) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut post_to_update = canister_data.get_post(&id).unwrap().clone();

        post_to_update.update_status(status.into());
        canister_data.add_post(post_to_update)
    });
}
