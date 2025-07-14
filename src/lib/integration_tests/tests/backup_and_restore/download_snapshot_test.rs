use std::{collections::HashMap, time::Duration};

use candid::{encode_one, Encode, Principal};
use pocket_ic::{PocketIc, WasmResult};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            arg::{IndividualUserTemplateInitArgs, PlaceBetArg},
            error::{BetOnCurrentlyViewingPostError, GetPostsOfUserProfileError},
            hot_or_not::{BetDirection, BettingStatus, PlacedBetDetail},
            post::{PostDetailsForFrontend, PostDetailsFromFrontend},
            profile::UserProfileDetailsForFrontend,
        },
        platform_orchestrator::types::args::PlatformOrchestratorInitArgs,
        user_index::types::UpgradeStatus,
    },
    common::{
        types::{known_principal::KnownPrincipalType, utility_token::token_event::TokenEvent},
        utils::default_pump_dump_onboarding_reward,
    },
    constant::RECLAIM_CANISTER_PRINCIPAL_ID,
    types::canister_specific::individual_user_template::error_types::GetUserUtilityTokenTransactionHistoryError,
};
use test_utils::setup::{
    env::pocket_ic_env::get_new_pocket_ic_env,
    test_constants::{
        get_mock_user_alice_principal_id, get_mock_user_bob_principal_id,
        get_mock_user_charlie_principal_id, get_mock_user_dan_principal_id,
    },
};

const INDIVIDUAL_TEMPLATE_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/individual_user_template.wasm.gz";

fn individual_template_canister_wasm() -> Vec<u8> {
    std::fs::read(INDIVIDUAL_TEMPLATE_WASM_PATH).unwrap()
}

// #[cfg(feature = "bet_details_heap_to_stable_mem_upgrade")]
#[test]
fn download_snapshot_test() {
    let (pic, known_principals) = get_new_pocket_ic_env();

    let platform_canister_id = known_principals
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let global_admin = known_principals
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pic.topology().get_app_subnets();

    let subnet_orchestrator_canister_id = pic
        .update_call(
            platform_canister_id,
            global_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[0]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    println!("subnet_orchestrator_canister_id: {:?}", subnet_orchestrator_canister_id);

    for _ in 0..50 {
        pic.tick()
    }

    let alice_principal_id = get_mock_user_alice_principal_id();
    let bob_principal_id = get_mock_user_bob_principal_id();
    let dan_principal_id = get_mock_user_dan_principal_id();

    let alice2_principal_id = Principal::self_authenticating([10]);
    let bob2_principal_id = Principal::self_authenticating([11]);
    let dan2_principal_id = Principal::self_authenticating([12]);

    let post_cache_canister_id = pic.create_canister();
    pic.add_cycles(post_cache_canister_id, 2_000_000_000_000);

    println!("post_cache_canister_id: {:?}", post_cache_canister_id);
    // Init individual template canister - alice

    let alice_individual_template_canister_id = pic
        .update_call(
            subnet_orchestrator_canister_id,
            alice_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();

    println!("alice_individual_template_canister_id: {:?}", alice_individual_template_canister_id);
    // Init individual template canister - bob

    let bob_individual_template_canister_id = pic
        .update_call(
            subnet_orchestrator_canister_id,
            bob_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();

    println!("bob_individual_template_canister_id: {:?}", bob_individual_template_canister_id);
    // Init individual template canister - dan

    let dan_individual_template_canister_id = pic
        .update_call(
            subnet_orchestrator_canister_id,
            dan_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();

    println!("dan_individual_template_canister_id: {:?}", dan_individual_template_canister_id);

    // Create posts
    // Alice creates a post
    let alice_post_1 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#1234".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res1 = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(alice_post_1).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    println!("res1: {:?}", res1);

    let alice_post_2 = PostDetailsFromFrontend {
        is_nsfw: false,
        description: "This is a fun video to watch 2".to_string(),
        hashtags: vec!["fun".to_string(), "video".to_string()],
        video_uid: "abcd#12345".to_string(),
        creator_consent_for_inclusion_in_hot_or_not: true,
    };
    let res2 = pic
        .update_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "add_post_v2",
            encode_one(alice_post_2).unwrap(),
        )
        .map(|reply_payload| {
            let newly_created_post_id_result: Result<u64, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 add_post failed\n"),
            };
            newly_created_post_id_result.unwrap()
        })
        .unwrap();

    println!("res2: {:?}", res2);

    // Top up Bob's account
    let reward = pic.update_call(
        bob_individual_template_canister_id,
        global_admin,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    println!("reward: {:?}", reward);

    // Top up Dan's account
    let reward = pic.update_call(
        dan_individual_template_canister_id,
        global_admin,
        "get_rewarded_for_signing_up",
        encode_one(()).unwrap(),
    );

    println!("reward: {:?}", reward);

    // Upgrade canister
    // Individual template canisters
    let individual_template_wasm_bytes = individual_template_canister_wasm();

    pic.update_call(
        subnet_orchestrator_canister_id,
        platform_canister_id,
        "start_upgrades_for_individual_canisters",
        candid::encode_args(("v2.2.2".to_owned(), individual_template_wasm_bytes.to_vec()))
            .unwrap(),
    )
    .map(|res| {
        let result: String = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("start upgrades for individual canister failed"),
        };
        result
    })
    .unwrap();

    for _ in 0..110 {
        pic.tick()
    }

    //Check version Installed
    let last_upgrade_status: UpgradeStatus = pic
        .query_call(
            subnet_orchestrator_canister_id,
            Principal::anonymous(),
            "get_index_details_last_upgrade_status",
            candid::encode_one(()).unwrap(),
        )
        .map(|res| {
            let upgrade_status: UpgradeStatus = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            upgrade_status
        })
        .unwrap();

    assert!(last_upgrade_status.version.eq("v2.2.2"));
    assert_eq!(last_upgrade_status.failed_canister_ids.len(), 0);

    // Save snapshot
    let reclaim_principal_id = Principal::from_text(RECLAIM_CANISTER_PRINCIPAL_ID).unwrap();

    let alice_snap_len = pic
        .update_call(
            alice_individual_template_canister_id,
            reclaim_principal_id,
            "save_snapshot_json",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: u32 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 save_snapshot_json failed\n"),
            };
            bet_status
        })
        .unwrap();
    println!("save_snapshot_json len: {:?}", alice_snap_len);

    let bob_snap_len = pic
        .update_call(
            bob_individual_template_canister_id,
            reclaim_principal_id,
            "save_snapshot_json",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: u32 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 save_snapshot_json failed\n"),
            };
            bet_status
        })
        .unwrap();
    println!("save_snapshot_json len: {:?}", bob_snap_len);

    let dan_snap_len = pic
        .update_call(
            dan_individual_template_canister_id,
            reclaim_principal_id,
            "save_snapshot_json",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let bet_status: u32 = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 save_snapshot_json failed\n"),
            };
            bet_status
        })
        .unwrap();
    println!("save_snapshot_json len: {:?}", dan_snap_len);

    // Download Snapshots
    //
    let mut alice_snapshot = vec![];
    let CHUNK_SIZE = 50;
    let num_iters = (alice_snap_len as f64 / CHUNK_SIZE as f64).ceil() as u32;
    for i in 0..num_iters {
        let start = i * CHUNK_SIZE;
        let mut end = (i + 1) * CHUNK_SIZE;
        if end > alice_snap_len {
            end = alice_snap_len;
        }
        println!("i {} start {} end-start {}", i, start, end - start);

        let chunk = pic
            .update_call(
                alice_individual_template_canister_id,
                reclaim_principal_id,
                "download_snapshot",
                candid::encode_args((start as u64, (end - start) as u64)).unwrap(),
            )
            .map(|reply_payload| {
                let payload: Vec<u8> = match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 download_snapshot failed\n"),
                };
                payload
            })
            .unwrap();
        println!("iter {} chunk: {:?}", i, chunk.len());
        alice_snapshot.extend(chunk);
    }
    println!("alice_snapshot: {:?}", alice_snapshot.len());

    let bob_snapshot = pic
        .update_call(
            bob_individual_template_canister_id,
            reclaim_principal_id,
            "download_snapshot",
            candid::encode_args((0 as u64, bob_snap_len as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: Vec<u8> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 download_snapshot failed\n"),
            };
            payload
        })
        .unwrap();

    println!("bob_snapshot: {:?}", bob_snapshot.len());

    let dan_snapshot = pic
        .update_call(
            dan_individual_template_canister_id,
            reclaim_principal_id,
            "download_snapshot",
            candid::encode_args((0 as u64, dan_snap_len as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: Vec<u8> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 download_snapshot failed\n"),
            };
            payload
        })
        .unwrap();
    println!("dan_snapshot: {:?}", dan_snapshot.len());

    // Clear snapshot

    let _: () = pic
        .update_call(
            alice_individual_template_canister_id,
            reclaim_principal_id,
            "clear_snapshot",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
        })
        .unwrap();

    let res = pic
        .update_call(
            alice_individual_template_canister_id,
            reclaim_principal_id,
            "download_snapshot",
            candid::encode_args((0 as u64, 10 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: Vec<u8> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        });
    println!("Expected err res: {:?}", res);
    assert_eq!(res.is_err(), true);

    // Query Alice canister for info

    let fres1 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice profile: {:?}", fres1);

    let fres2 = pic
        .query_call(
            alice_individual_template_canister_id,
            alice_principal_id,
            "get_posts_of_this_user_profile_with_pagination_cursor",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile failed\n"),
                };
            profile.unwrap()
        })
        .unwrap();
    println!("Alice posts: {:?}", fres2);

    // Stop canisters
    match pic.stop_canister(
        alice_individual_template_canister_id,
        Some(subnet_orchestrator_canister_id),
    ) {
        Ok(_) => println!("Alice stopped"),
        Err(e) => println!("Alice stop error: {:?}", e),
    };

    match pic.stop_canister(
        bob_individual_template_canister_id,
        Some(subnet_orchestrator_canister_id),
    ) {
        Ok(_) => println!("Bob stopped"),
        Err(e) => println!("Bob stop error: {:?}", e),
    };

    match pic.stop_canister(
        dan_individual_template_canister_id,
        Some(subnet_orchestrator_canister_id),
    ) {
        Ok(_) => println!("Dan stopped"),
        Err(e) => println!("Dan stop error: {:?}", e),
    };

    // Init 2nd gen canisters
    /// Alice 2
    let alice2_individual_template_canister_id = pic
        .update_call(
            subnet_orchestrator_canister_id,
            alice2_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();

    /// Bob 2
    let bob2_individual_template_canister_id = pic
        .update_call(
            subnet_orchestrator_canister_id,
            bob2_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();

    /// Dan 2
    let dan2_individual_template_canister_id = pic
        .update_call(
            subnet_orchestrator_canister_id,
            dan2_principal_id,
            "get_requester_principals_canister_id_create_if_not_exists",
            candid::encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let response: Result<Principal, String> = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get requester principals canister id failed\n"),
            };
            response
        })
        .unwrap()
        .unwrap();
    // Check new canisters

    let res = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice2_principal_id,
            "get_posts_of_this_user_profile_with_pagination_cursor",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile failed\n"),
                };
            profile
        })
        .unwrap();
    println!("Alice 2 posts: {:?}", res);
    assert!(res.is_err());

    // Restore state
    let res: () = pic
        .update_call(
            alice2_individual_template_canister_id,
            reclaim_principal_id,
            "receive_and_save_snaphot",
            candid::encode_args((0 as u64, alice_snapshot)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let res: () = pic
        .update_call(
            bob2_individual_template_canister_id,
            reclaim_principal_id,
            "receive_and_save_snaphot",
            candid::encode_args((0 as u64, bob_snapshot)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let res: () = pic
        .update_call(
            dan2_individual_template_canister_id,
            reclaim_principal_id,
            "receive_and_save_snaphot",
            candid::encode_args((0 as u64, dan_snapshot)).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    // Load snapshots

    let res: () = pic
        .update_call(
            alice2_individual_template_canister_id,
            reclaim_principal_id,
            "load_snapshot",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let res: () = pic
        .update_call(
            bob2_individual_template_canister_id,
            reclaim_principal_id,
            "load_snapshot",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    let res: () = pic
        .update_call(
            dan2_individual_template_canister_id,
            reclaim_principal_id,
            "load_snapshot",
            candid::encode_args(()).unwrap(),
        )
        .map(|reply_payload| {
            let payload: _ = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 place_bet failed\n"),
            };
            payload
        })
        .unwrap();

    // Query Alice canister for info

    let fres1_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice2_principal_id,
            "get_profile_details",
            encode_one(()).unwrap(),
        )
        .map(|reply_payload| {
            let profile: UserProfileDetailsForFrontend = match reply_payload {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("\n🛑 get_profile failed\n"),
            };
            profile
        })
        .unwrap();
    println!("Alice profile: {:?}", fres1_1);
    assert_eq!(fres1_1, fres1);

    let fres2_1 = pic
        .query_call(
            alice2_individual_template_canister_id,
            alice2_principal_id,
            "get_posts_of_this_user_profile_with_pagination_cursor",
            candid::encode_args((0 as u64, 5 as u64)).unwrap(),
        )
        .map(|reply_payload| {
            let profile: Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> =
                match reply_payload {
                    WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                    _ => panic!("\n🛑 get_profile failed\n"),
                };
            profile.unwrap()
        })
        .unwrap();
    println!("Alice posts: {:?}", fres2_1);
    assert_eq!(fres2_1, fres2);
}

#[test]
fn all_canister_snapshot_tests() {
    let (pocket_ic, known_principal) = get_new_pocket_ic_env();
    let platform_canister_id = known_principal
        .get(&KnownPrincipalType::CanisterIdPlatformOrchestrator)
        .cloned()
        .unwrap();

    let super_admin = known_principal
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .cloned()
        .unwrap();

    let application_subnets = pocket_ic.topology().get_app_subnets();

    let charlie_global_admin = get_mock_user_charlie_principal_id();

    pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "add_principal_as_global_admin",
            candid::encode_one(charlie_global_admin).unwrap(),
        )
        .unwrap();

    let user_index_canister_id: Principal = pocket_ic
        .update_call(
            platform_canister_id,
            super_admin,
            "provision_subnet_orchestrator_canister",
            candid::encode_one(application_subnets[1]).unwrap(),
        )
        .map(|res| {
            let canister_id_result: Result<Principal, String> = match res {
                WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
                _ => panic!("Canister call failed"),
            };
            canister_id_result.unwrap()
        })
        .unwrap();

    for i in 0..50 {
        pocket_ic.tick();
    }

    // upgrade pf_orch

    let platform_orchestrator_init_args = PlatformOrchestratorInitArgs {
        version: "v1.0.0".into(),
    };
    pocket_ic
        .upgrade_canister(
            platform_canister_id,
            pf_orch_canister_wasm(),
            candid::encode_one(platform_orchestrator_init_args).unwrap(),
            Some(super_admin),
        )
        .unwrap();
    for i in 0..20 {
        pocket_ic.tick();
    }

    let reclaim_principal_id = Principal::from_text(RECLAIM_CANISTER_PRINCIPAL_ID).unwrap();

    let response = pocket_ic
        .update_call(
            platform_canister_id,
            reclaim_principal_id,
            "save_snapshot_json",
            Encode!().unwrap(),
        )
        .unwrap();
    let snapshot_len: u32 = match response {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 save_snapshot_json failed for platform orchestrator\n"),
    };

    let mut data: Vec<u8> = Vec::new();
    let mut offset: u64 = 0;
    let chunk_size = 100_000;

    while offset < snapshot_len as u64 {
        let length = std::cmp::min(chunk_size, snapshot_len as u64 - offset);

        let response = pocket_ic
            .query_call(
                platform_canister_id,
                reclaim_principal_id,
                "download_snapshot",
                Encode!(&offset, &length).unwrap(),
            )
            .unwrap();
        let chunk: Vec<u8> = match response {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 download_snapshot platform orchestrator failed for {offset}\n"),
        };

        data.extend_from_slice(&chunk);
        offset += length;
    }

    println!("data: {}", std::str::from_utf8(&data).unwrap());

    let snapshot_len = data.len() as u64;
    let mut offset: u64 = 0;
    let chunk_size: u64 = 100_000;

    while offset < snapshot_len {
        let length = std::cmp::min(chunk_size, snapshot_len - offset);
        let chunk = &data[(offset as usize)..((offset + length) as usize)];

        if pocket_ic
            .update_call(
                platform_canister_id,
                reclaim_principal_id,
                "receive_and_save_snaphot",
                Encode!(&offset, &chunk).unwrap(),
            )
            .is_err()
        {
            panic!("\n🛑 receive_and_save_snaphot failed for platform orchestrator\n")
        };
        offset += length;
    }

    pocket_ic
        .update_call(
            platform_canister_id,
            reclaim_principal_id,
            "load_snapshot",
            Encode!(&()).unwrap(),
        )
        .unwrap();

    let response = pocket_ic
        .update_call(
            user_index_canister_id,
            reclaim_principal_id,
            "save_snapshot_json",
            Encode!().unwrap(),
        )
        .unwrap();
    let snapshot_len: u32 = match response {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 save_snapshot failed for user index\n"),
    };

    let mut data: Vec<u8> = Vec::new();
    let mut offset: u64 = 0;
    let chunk_size = 100_000;

    while offset < snapshot_len as u64 {
        let length = std::cmp::min(chunk_size, snapshot_len as u64 - offset);

        let response = pocket_ic
            .query_call(
                user_index_canister_id,
                reclaim_principal_id,
                "download_snapshot",
                Encode!(&offset, &length).unwrap(),
            )
            .unwrap();
        let chunk: Vec<u8> = match response {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 download_snapshot failed for user index\n"),
        };

        data.extend_from_slice(&chunk);
        offset += length;
    }

    println!("data: {}", std::str::from_utf8(&data).unwrap());

    let snapshot_len = data.len() as u64;
    let mut offset: u64 = 0;
    let chunk_size: u64 = 100_000;

    while offset < snapshot_len {
        let length = std::cmp::min(chunk_size, snapshot_len - offset);
        let chunk = &data[(offset as usize)..((offset + length) as usize)];

        if pocket_ic
            .update_call(
                user_index_canister_id,
                reclaim_principal_id,
                "receive_and_save_snaphot",
                Encode!(&offset, &chunk).unwrap(),
            )
            .is_err()
        {
            panic!("\n🛑receive_and_save_snaphot failed for user index\n")
        };
        offset += length;
    }

    if pocket_ic
        .update_call(
            user_index_canister_id,
            reclaim_principal_id,
            "load_snapshot",
            Encode!(&()).unwrap(),
        )
        .is_err()
    {
        panic!("\n🛑Load snapshot failed for user index\n")
    };
}

const PF_ORCH_WASM_PATH: &str =
    "../../../target/wasm32-unknown-unknown/release/platform_orchestrator.wasm.gz";

fn pf_orch_canister_wasm() -> Vec<u8> {
    std::fs::read(PF_ORCH_WASM_PATH).unwrap()
}
