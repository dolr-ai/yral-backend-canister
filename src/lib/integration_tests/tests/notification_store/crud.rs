use candid::Encode;
use pocket_ic::WasmResult;
use shared_utils::{canister_specific::notification_store::types::{error::NotificationStoreError, notification::{NotificationData, NotificationType, VideoUploadPayload}}, common::types::known_principal::KnownPrincipalType};
use test_utils::setup::{env::pocket_ic_env::{get_new_pocket_ic_env, get_new_pocket_ic_env_with_service_canisters_provisioned}, test_constants::get_mock_user_alice_principal_id};

#[test]
fn test_crud() {
    let (pic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let notification_store_canister_id = service_canisters.notification_store_canister_id;

    let alice_principal = get_mock_user_alice_principal_id();
    let res = pic.update_call(notification_store_canister_id, alice_principal, "add_notification", candid::encode_one(NotificationType::VideoUpload(VideoUploadPayload {
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

    let res = pic.update_call(notification_store_canister_id, alice_principal, "mark_notification_as_read", candid::encode_one(0u64).unwrap()).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 mark notification as read failed\n"),
    };
    res.unwrap();

    let notifications = pic.query_call(notification_store_canister_id, alice_principal, "get_notifications", Encode!(&10u64, &0u64).unwrap()).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };

    assert_eq!(notifications.len(), 1);
    assert!(notifications[0].read);
}

#[test]
fn test_increment_notification_id() {
    let (pic, known_principals) = get_new_pocket_ic_env();

    let notification_store_canister_id = known_principals
        .get(&KnownPrincipalType::CanisterIdNotificationStore)
        .cloned()
        .unwrap();

    let alice_principal = get_mock_user_alice_principal_id();
    let res = pic.update_call(notification_store_canister_id, alice_principal, "add_notification", candid::encode_one(NotificationType::VideoUpload(VideoUploadPayload {
        video_uid: 1,
    })).unwrap()).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add notification failed\n"),
    };
    res.unwrap();

    let res = pic.update_call(notification_store_canister_id, alice_principal, "add_notification", candid::encode_one(NotificationType::VideoUpload(VideoUploadPayload {
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