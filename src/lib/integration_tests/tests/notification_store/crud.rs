use candid::Encode;
use pocket_ic::WasmResult;
use shared_utils::{canister_specific::notification_store::types::{error::NotificationStoreError, notification::{NotificationData, NotificationType, VideoUploadPayload}}, common::types::known_principal::KnownPrincipalType};
use test_utils::setup::{env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned, test_constants::get_mock_user_alice_principal_id};

#[test]
fn test_crud() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let notification_store_canister_id = service_canisters.notification_store_canister_id;

    let alice_principal = get_mock_user_alice_principal_id();
    let res = pic.update_call(notification_store_canister_id, alice_principal, "add_notification", Encode!(&alice_principal, &NotificationType::VideoUpload(VideoUploadPayload {
        video_uid: 1,
    })).unwrap()).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add notification failed\n"),
    };
    res.unwrap();

    let notifications = pic.query_call(notification_store_canister_id, alice_principal, "get_notifications", Encode!(&10u64, &0u64).unwrap()).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };

    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].payload, NotificationType::VideoUpload(VideoUploadPayload {
        video_uid: 1,
    }));

    let res = pic.update_call(notification_store_canister_id, alice_principal, "set_notification_panel_viewed", candid::encode_args(()).unwrap()).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 set notification panel viewed failed\n"),
    };
    res.unwrap();
}

#[test]
fn test_increment_notification_id() {
    let (pic, known_principals) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let notification_store_canister_id = known_principals.notification_store_canister_id;

    let alice_principal = get_mock_user_alice_principal_id();
    let res = pic.update_call(notification_store_canister_id, alice_principal, "add_notification", Encode!(&alice_principal, &NotificationType::VideoUpload(VideoUploadPayload {
        video_uid: 1,
    })).unwrap()).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add notification failed\n"),
    };
    res.unwrap();

    let res = pic.update_call(notification_store_canister_id, alice_principal, "add_notification", Encode!(&alice_principal, &NotificationType::VideoUpload(VideoUploadPayload {
        video_uid: 2,
    })).unwrap()).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add notification failed\n"),
    };
    res.unwrap();

    let notifications = pic.query_call(notification_store_canister_id, alice_principal, "get_notifications", Encode!(&10u64, &0u64).unwrap()).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };

    assert_eq!(notifications.len(), 2);
    assert_eq!(notifications[0].notification_id, 0);
    assert_eq!(notifications[1].notification_id, 1);
}

#[test]
fn test_auto_prune_notifications() {
    let (pic, known_principals) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let notification_store_canister = known_principals.notification_store_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    
    for i in 0..1000 {
        let res = pic.update_call(
            notification_store_canister, 
            alice_principal, 
            "add_notification", 
            Encode!(&alice_principal, &NotificationType::VideoUpload(VideoUploadPayload {
                video_uid: i,
            })).unwrap()
        ).unwrap();
        let res: Result<(), NotificationStoreError> = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 add notification failed\n"),
        };
        res.unwrap();
    }
    
    let notifications = pic.query_call(
        notification_store_canister, 
        alice_principal, 
        "get_notifications", 
        Encode!(&1000u64, &0u64).unwrap()
    ).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };
    
    assert_eq!(notifications.len(), 500);
    
    assert_eq!(notifications[0].notification_id, 0);
    assert_eq!(notifications[499].notification_id, 499);
    
    // Verify we have the most recent notifications (video_uid 500-999)
    // The oldest remaining notification should have video_uid 500
    if let NotificationType::VideoUpload(payload) = &notifications[0].payload {
        assert_eq!(payload.video_uid, 500);
    } else {
        panic!("Expected VideoUpload notification type");
    }
    
    // The newest notification should have video_uid 999
    if let NotificationType::VideoUpload(payload) = &notifications[499].payload {
        assert_eq!(payload.video_uid, 999);
    } else {
        panic!("Expected VideoUpload notification type");
    }
}

#[test]
fn test_notification_panel_viewed() {
    let (pic, known_principals) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let notification_store_canister = known_principals.notification_store_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    
    let res = pic.update_call(
        notification_store_canister,
        alice_principal,
        "set_notification_panel_viewed",
        candid::encode_args(()).unwrap()
    ).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 set notification panel viewed failed\n"),
    };
    res.unwrap();
    
    let res = pic.update_call(
        notification_store_canister,
        alice_principal,
        "add_notification",
        Encode!(&alice_principal, &NotificationType::VideoUpload(VideoUploadPayload {
            video_uid: 1,
        })).unwrap()
    ).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add notification failed\n"),
    };
    res.unwrap();
    
    let res = pic.update_call(
        notification_store_canister,
        alice_principal,
        "set_notification_panel_viewed",
        candid::encode_args(()).unwrap()
    ).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 set notification panel viewed failed\n"),
    };
    res.unwrap();
    
    let notifications = pic.query_call(
        notification_store_canister,
        alice_principal,
        "get_notifications",
        Encode!(&10u64, &0u64).unwrap()
    ).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };
    
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].payload, NotificationType::VideoUpload(VideoUploadPayload {
        video_uid: 1,
    }));
}

#[test]
fn test_pagination() {
    let (pic, known_principals) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let notification_store_canister = known_principals.notification_store_canister_id;
    let alice_principal = get_mock_user_alice_principal_id();
    
    for i in 0..10 {
        let res = pic.update_call(
            notification_store_canister,
            alice_principal,
            "add_notification",
            Encode!(&alice_principal, &NotificationType::VideoUpload(VideoUploadPayload {
                video_uid: i,
            })).unwrap()
        ).unwrap();
        let res: Result<(), NotificationStoreError> = match res {
            WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
            _ => panic!("\n🛑 add notification failed\n"),
        };
        res.unwrap();
    }
    
    let notifications = pic.query_call(
        notification_store_canister,
        alice_principal,
        "get_notifications",
        Encode!(&5u64, &0u64).unwrap()
    ).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };
    
    assert_eq!(notifications.len(), 5);
    assert_eq!(notifications[0].notification_id, 0);
    assert_eq!(notifications[4].notification_id, 4);
    
    let notifications = pic.query_call(
        notification_store_canister,
        alice_principal,
        "get_notifications",
        Encode!(&5u64, &5u64).unwrap()
    ).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };
    
    assert_eq!(notifications.len(), 5);
    assert_eq!(notifications[0].notification_id, 5);
    assert_eq!(notifications[4].notification_id, 9);
    
    let notifications = pic.query_call(
        notification_store_canister,
        alice_principal,
        "get_notifications",
        Encode!(&5u64, &15u64).unwrap()
    ).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };
    
    assert_eq!(notifications.len(), 0);
}