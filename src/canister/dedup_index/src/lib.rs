mod canister_lifecycle;
mod data_model;

use std::cell::RefCell;

use data_model::CanisterData;
use ic_cdk::{export_candid, update};
use ic_cdk_macros::query;
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
    DEDUP_INDEX.with_borrow_mut(|CanisterData { index, .. }| index.contains_key(&video_hash))
}

export_candid!();
