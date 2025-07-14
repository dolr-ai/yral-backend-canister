use pocket_ic::WasmResult;
use shared_utils::{canister_specific::notification_store::types::{error::NotificationStoreError, notification::{NotificationData, NotificationType, VideoUploadPayload}}, common::types::known_principal::KnownPrincipalType};
use test_utils::setup::{env::pocket_ic_env::get_new_pocket_ic_env, test_constants::{get_mock_user_alice_principal_id, get_mock_user_charlie_principal_id}};

#[test]
fn test_guard() {
    let (pic, known_principals) = get_new_pocket_ic_env();

    let notification_store_canister_id = known_principals
        .get(&KnownPrincipalType::CanisterIdNotificationStore)
        .cloned()
        .unwrap();

    let alice_principal = get_mock_user_alice_principal_id();
    let res = pic.update_call(notification_store_canister_id, alice_principal, "add_notification", candid::encode_one(NotificationType::VideoUpload(VideoUploadPayload {
        video_id: 1,
    })).unwrap()).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add notification failed\n"),
    };
    res.unwrap();

    let notifications = pic.query_call(notification_store_canister_id, alice_principal, "get_notifications", candid::encode_one((10, 0, )).unwrap()).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };

    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].payload, NotificationType::VideoUpload(VideoUploadPayload {
        video_id: 1,
    }));

    let charlie_principal = get_mock_user_charlie_principal_id();
    let guard_res = pic.query_call(notification_store_canister_id, charlie_principal, "get_notifications", candid::encode_one((10, 0, )).unwrap()).unwrap();
    let guard_res: Vec<NotificationData> = match guard_res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };

    assert_eq!(guard_res.len(), 0);
}