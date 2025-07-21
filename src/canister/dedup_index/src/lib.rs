mod canister_lifecycle;
mod data_model;

use std::cell::RefCell;

use data_model::CanisterData;
use ic_cdk::{export_candid, update};
use ic_cdk_macros::query;
use shared_utils::canister_specific::dedup_index::{
    LISTING_SIZE_LIMIT_EXCLUSIVE, ListArgs, ListError, VideoHash,
};
use shared_utils::service::ServiceInitArgs;
use shared_utils::{
    canister_specific::dedup_index::{Video, VideoId, Videos},
    common::utils::permissions::is_caller_controller_or_global_admin,
};

thread_local! {
    static DEDUP_INDEX: RefCell<CanisterData> = RefCell::default();
}

#[update(guard = "is_caller_controller_or_global_admin")]
fn add_video_to_index(video_id: VideoId, (video_hash, timestamp): Video) {
    DEDUP_INDEX.with_borrow_mut(|CanisterData { index, .. }| {
        let Some(ref mut videos) = index.get(&video_id) else {
            index.insert(video_hash, Videos([(video_id, timestamp)].into()));
            return;
        };

        videos.insert((video_id, timestamp));
    })
}

#[query]
fn is_duplicate(video_hash: String) -> bool {
    DEDUP_INDEX.with_borrow(|CanisterData { index, .. }| index.contains_key(&video_hash))
}

#[query]
fn unique_hash_count() -> u64 {
    DEDUP_INDEX.with_borrow(|CanisterData { index, .. }| index.len())
}

#[query]
fn list_hashes(ListArgs { page, size }: ListArgs) -> Result<Vec<(VideoHash, Videos)>, ListError> {
    let start_idx = page.checked_mul(size).ok_or(ListError::WillOverflow)?;
    // allow admin to fetch as many items, and limit for everyone else. usefull when inspecting with
    if size > LISTING_SIZE_LIMIT_EXCLUSIVE && is_caller_controller_or_global_admin().is_err() {
        return Err(ListError::SizeNotAllowed);
    }

    DEDUP_INDEX.with_borrow(|CanisterData { index, .. }| {
        if start_idx as u64 >= index.len() {
            return Err(ListError::PageOutOfRange);
        }
        Ok(index.iter().skip(start_idx).take(size).collect())
    })
}

export_candid!();
