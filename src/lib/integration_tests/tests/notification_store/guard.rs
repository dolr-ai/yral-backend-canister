use candid::Encode;
use pocket_ic::WasmResult;
use shared_utils::{canister_specific::notification_store::types::{error::NotificationStoreError, notification::{NotificationData, NotificationType, VideoUploadPayload}}, common::types::known_principal::KnownPrincipalType};
use test_utils::setup::{env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned, test_constants::{get_mock_user_alice_principal_id, get_mock_user_charlie_principal_id}};

#[test]
fn test_notification_authorization() {
    let (pic, known_principals) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let notification_store_canister_id = known_principals.notification_store_canister_id;

    let alice_principal = get_mock_user_alice_principal_id();
    let res = pic.update_call(notification_store_canister_id, alice_principal, "add_notification", Encode!(&alice_principal, &NotificationType::VideoUpload(VideoUploadPayload {
        video_uid: 1.to_string(),
    })).unwrap()).unwrap();
    let res: Result<(), NotificationStoreError> = match res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 add notification failed\n"),
    };
    res.unwrap();

    let notifications = pic.query_call(notification_store_canister_id, alice_principal, "get_notifications", Encode!(&10u64, &0u64 ).unwrap()).unwrap();
    let notifications: Vec<NotificationData> = match notifications {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };

    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].payload, NotificationType::VideoUpload(VideoUploadPayload {
        video_uid: 1.to_string(),
    }));

    let charlie_principal = get_mock_user_charlie_principal_id();
    let guard_res = pic.query_call(notification_store_canister_id, charlie_principal, "get_notifications", Encode!(&10u64, &0u64 ).unwrap()).unwrap();
    let guard_res: Vec<NotificationData> = match guard_res {
        WasmResult::Reply(payload) => candid::decode_one(&payload).unwrap(),
        _ => panic!("\n🛑 get notifications failed\n"),
    };

    assert_eq!(guard_res.len(), 0);
}