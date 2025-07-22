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
