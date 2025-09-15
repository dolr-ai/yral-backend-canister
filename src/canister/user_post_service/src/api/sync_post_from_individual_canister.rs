use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::user_post_service::types::storage::Post,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
fn sync_post_from_individual_canister(post: Post) -> Option<Post> {
    CANISTER_DATA.with_borrow_mut(|canister_data| canister_data.add_post(post))
}
