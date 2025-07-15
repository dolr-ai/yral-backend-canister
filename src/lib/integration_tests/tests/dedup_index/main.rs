use std::time::SystemTime;

use candid::{decode_one, encode_one, Encode};
use pocket_ic::WasmResult;
use test_utils::setup::{
    env::pocket_ic_env::{
        get_new_pocket_ic_env_with_service_canisters_provisioned, ServiceCanisters,
    },
    test_constants::get_global_super_admin_principal_id,
};

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
