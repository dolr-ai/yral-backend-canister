use crate::CANISTER_DATA;
use ic_cdk_macros::query;
use shared_utils::{
    canister_specific::user_info_service::args::PostIdVideoUidMappingPaginationResult,
    common::utils::permissions::is_caller_controller,
};

#[query(guard = "is_caller_controller")]
fn get_post_id_video_uid_mapping_with_pagination(
    last_uuid_processed: Option<String>,
    limit: usize,
) -> PostIdVideoUidMappingPaginationResult {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.get_post_id_video_uid_mapping_with_pagination(limit, last_uuid_processed)
    })
}
