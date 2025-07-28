use std::time::SystemTime;

use candid::{decode_one, encode_one, Encode, Principal};
use pocket_ic::WasmResult;
use shared_utils::canister_specific::dedup_index::{
    ListArgs, ListError, VideoHash, Videos, LISTING_SIZE_LIMIT_EXCLUSIVE,
};
use test_utils::setup::{
    env::pocket_ic_env::{
        get_new_pocket_ic_env_with_service_canisters_provisioned, ServiceCanisters,
    },
    test_constants::get_global_super_admin_principal_id,
};

mod helper {
    use std::time::SystemTime;

    use candid::{decode_one, encode_one, Encode, Principal};
    use pocket_ic::{PocketIc, WasmResult};
    use shared_utils::canister_specific::dedup_index::{
        RemoveVideoIdArgs, VideoHash, VideoId, VideoListByHash,
    };

    #[inline]
    pub fn add_video(
        pic: &PocketIc,
        canister_id: Principal,
        sender: Principal,
        video_id: impl Into<VideoId>,
        video_hash: impl Into<VideoHash>,
    ) {
        let res = pic
            .update_call(
                canister_id,
                sender,
                "add_video_to_index",
                Encode!(&video_id.into(), &(video_hash.into(), SystemTime::now())).unwrap(),
            )
            .unwrap();

        if let WasmResult::Reject(reason) = res {
            panic!("add video call rejected: {reason}")
        }
    }

    #[inline]
    pub fn remove_video(
        pic: &PocketIc,
        canister_id: Principal,
        sender: Principal,
        video_id: impl Into<VideoId>,
        video_hash: impl Into<VideoHash>,
    ) {
        pic.update_call(
            canister_id,
            sender,
            "remove_video_id",
            Encode!(&RemoveVideoIdArgs {
                video_id: video_id.into(),
                video_hash: video_hash.into()
            })
            .unwrap(),
        )
        .unwrap();
    }

    #[inline]
    pub fn get_video_by_hash(
        pic: &PocketIc,
        canister_id: Principal,
        sender: Principal,
        video_hash: impl Into<VideoHash>,
    ) -> Option<VideoListByHash> {
        let res = pic
            .query_call(
                canister_id,
                sender,
                "get_videos_for_hash",
                encode_one(video_hash.into()).unwrap(),
            )
            .unwrap();

        match res {
            WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
            _ => panic!("couldn't query for videos by hash"),
        }
    }
}

#[test]
fn test_dedup_index_deletion() {
    let (
        pic,
        ServiceCanisters {
            dedup_index_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let sender = get_global_super_admin_principal_id();

    helper::add_video(&pic, canister_id, sender, "video_1", "video_hash");
    helper::add_video(&pic, canister_id, sender, "video_2", "video_hash");

    let list = helper::get_video_by_hash(&pic, canister_id, sender, "video_hash")
        .expect("list to be returned");

    assert_eq!(list.len(), 2);

    helper::remove_video(&pic, canister_id, sender, "video_1", "video_hash");

    let list = helper::get_video_by_hash(&pic, canister_id, sender, "video_hash")
        .expect("list to be returned");

    assert_eq!(list.len(), 1);

    assert_eq!(list.first().unwrap().0.as_str(), "video_2");
}

#[test]
fn test_dedup_index_multi_insertion() {
    let (
        pic,
        ServiceCanisters {
            dedup_index_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let sender = get_global_super_admin_principal_id();

    helper::add_video(&pic, canister_id, sender, "video_1", "video_hash");
    helper::add_video(&pic, canister_id, sender, "video_2", "video_hash");

    let list_by_hash = helper::get_video_by_hash(&pic, canister_id, sender, "video_hash")
        .expect("a list must exist");

    assert_eq!(list_by_hash.len(), 2);

    let ids: Vec<_> = list_by_hash.into_iter().map(|(id, _)| id).collect();

    assert!(ids.contains(&"video_1".to_string()), "video_1 must exist");

    assert!(ids.contains(&"video_2".to_string()), "video_2 must exist");
}

#[test]
fn test_dedup_index_listing() {
    let (
        pic,
        ServiceCanisters {
            dedup_index_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let sender = get_global_super_admin_principal_id();

    const TOTAL: usize = 5;
    for i in 0..TOTAL {
        // this test can assume insertion succeeds
        pic.update_call(
            canister_id,
            sender,
            "add_video_to_index",
            Encode!(
                &"some_video_id".to_string(),
                &(i.to_string(), SystemTime::now())
            )
            .unwrap(),
        )
        .unwrap();
    }

    // with size=2, there should be three pages of len 2 + 2 + 1
    const SIZE: usize = 2;
    let fetch_page = |page: usize| {
        let res = pic
            .query_call(
                canister_id,
                sender,
                "list_hashes",
                encode_one(ListArgs { page, size: SIZE }).unwrap(),
            )
            .unwrap();

        let list: Result<Vec<(VideoHash, Videos)>, ListError> = match res {
            WasmResult::Reply(payload) => decode_one(&payload).unwrap(),
            _ => panic!("Couldn't query for listing hashes"),
        };

        list
    };

    if !fetch_page(0).is_ok_and(|l| l.len() == 2) {
        panic!("unexpected result when listing page 0 of hashes");
    }
    if !fetch_page(1).is_ok_and(|l| l.len() == 2) {
        panic!("unexpected result when listing page 1 of hashes");
    }
    if !fetch_page(2).is_ok_and(|l| l.len() == 1) {
        panic!("unexpected result when listing page 2 of hashes");
    }
}

#[test]
fn test_dedup_index_listing_size_limit() {
    let (
        pic,
        ServiceCanisters {
            dedup_index_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let sender = Principal::anonymous();

    let res = pic
        .query_call(
            canister_id,
            sender,
            "list_hashes",
            encode_one(ListArgs {
                page: 0,
                size: LISTING_SIZE_LIMIT_EXCLUSIVE + 1,
            })
            .unwrap(),
        )
        .unwrap();

    let list: Result<Vec<(VideoHash, Videos)>, ListError> = match res {
        WasmResult::Reply(payload) => decode_one(&payload).unwrap(),
        _ => panic!("Couldn't query for listing hashes"),
    };

    if !matches!(list, Err(ListError::SizeNotAllowed)) {
        panic!("non admins should not be able to list more than {LISTING_SIZE_LIMIT_EXCLUSIVE} items per call");
    }

    let res = pic
        .query_call(
            canister_id,
            get_global_super_admin_principal_id(),
            "list_hashes",
            encode_one(ListArgs {
                page: 0,
                size: LISTING_SIZE_LIMIT_EXCLUSIVE + 1,
            })
            .unwrap(),
        )
        .unwrap();

    let list: Result<Vec<(VideoHash, Videos)>, ListError> = match res {
        WasmResult::Reply(payload) => decode_one(&payload).unwrap(),
        _ => panic!("Couldn't query for listing hashes"),
    };

    if matches!(list, Err(ListError::SizeNotAllowed)) {
        panic!("no size limit should be imposed on the admin");
    }
}

#[test]
fn test_dedup_index_insertion_and_retrieval() {
    let (
        pic,
        ServiceCanisters {
            dedup_index_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let sender = get_global_super_admin_principal_id();

    const HASH: &str = "myuniquehashstring";

    let res = pic
        .query_call(
            canister_id,
            sender,
            "is_duplicate",
            encode_one(HASH).unwrap(),
        )
        .unwrap();
    // should be false because hash doesn't exist
    let is_duplicate: bool = match res {
        WasmResult::Reply(payload) => decode_one(&payload).unwrap(),
        _ => panic!("Couldn't query for duplication"),
    };

    assert!(!is_duplicate);

    let res = pic
        .update_call(
            canister_id,
            sender,
            "add_video_to_index",
            Encode!(&"some_video_id".to_string(), &(HASH, SystemTime::now())).unwrap(),
        )
        .unwrap();
    let _: () = match res {
        WasmResult::Reply(payload) => decode_one(&payload).unwrap(),
        _ => panic!("Couldn't add video to index"),
    };

    let res = pic
        .query_call(
            canister_id,
            sender,
            "is_duplicate",
            encode_one(HASH).unwrap(),
        )
        .unwrap();
    let is_duplicate: bool = match res {
        WasmResult::Reply(payload) => decode_one(&payload).unwrap(),
        _ => panic!("Couldn't query for duplication"),
    };

    assert!(is_duplicate);
}
