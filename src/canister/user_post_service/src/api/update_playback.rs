use shared_utils::canister_specific::user_post_service::types::storage::VideoSourceType;

use crate::CANISTER_DATA;

#[update(guard = "is_caller_global_admin")]
fn update_playback(
    post_id: String,
    source_type: VideoSourceType,
    source_url: String,
) -> Result<(), String> {
    CANISTER_DATA
        .with_borrow_mut(|data| data.update_playback_source(&post_id, source_type, source_url))
}
