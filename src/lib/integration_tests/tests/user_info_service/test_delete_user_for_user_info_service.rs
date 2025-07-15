use shared_utils::canister_specific::individual_user_template::types::{
    profile::UserProfileDetailsForFrontendV3, session::SessionType,
};
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
    get_mock_user_charlie_principal_id,
};

#[test]
fn test_delete_user_for_user_info_service() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_service_canister = service_canisters.user_info_service_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    let charlie_principal_id = get_mock_user_charlie_principal_id();

    // First, register the user
    let registration_result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        global_admin,
        "register_new_user",
        (charlie_principal_id,),
    )
    .expect("Failed to register new user");

    assert!(
        registration_result.is_ok(),
        "User registration failed: {:?}",
        registration_result
    );

    // Verify user exists by getting profile details
    let charlie_profile_details = query::<_, Result<UserProfileDetailsForFrontendV3, String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "get_profile_details",
        (),
    )
    .unwrap()
    .unwrap();

    assert_eq!(charlie_profile_details.principal_id, charlie_principal_id);

    // Verify user can get session type
    let session_type = query::<_, Result<SessionType, String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "get_session_type",
        (),
    )
    .expect("Failed to get session type")
    .unwrap();

    assert_eq!(session_type, SessionType::AnonymousSession);

    // Try to delete user as unauthorized user (should fail)
    let unauthorized_principal = get_mock_user_alice_principal_id();
    let unauthorized_delete_result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        unauthorized_principal,
        "delete_user_info",
        (charlie_principal_id,),
    )
    .expect("Failed to call delete_user_info");

    assert!(unauthorized_delete_result.is_err());
    assert!(unauthorized_delete_result
        .unwrap_err()
        .contains("Unauthorized"));

    // User should still exist after unauthorized deletion attempt
    let still_exists_profile = query::<_, Result<UserProfileDetailsForFrontendV3, String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "get_profile_details",
        (),
    )
    .unwrap()
    .unwrap();

    assert_eq!(still_exists_profile.principal_id, charlie_principal_id);

    // Delete user as global admin (should succeed)
    let admin_delete_result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        global_admin,
        "delete_user_info",
        (charlie_principal_id,),
    )
    .expect("Failed to delete user as admin");

    assert!(
        admin_delete_result.is_ok(),
        "Admin delete failed: {:?}",
        admin_delete_result
    );

    // Verify user no longer exists by trying to get profile details
    let profile_after_delete = query::<_, Result<UserProfileDetailsForFrontendV3, String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "get_profile_details",
        (),
    )
    .expect("Failed to call get_profile_details");

    assert!(profile_after_delete.is_err());
    assert!(profile_after_delete.unwrap_err().contains("User not found"));

    // Also verify session type query fails
    let session_type_after_delete = query::<_, Result<SessionType, String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "get_session_type",
        (),
    )
    .expect("Failed to call get_session_type");

    assert!(session_type_after_delete.is_err());
    assert!(session_type_after_delete
        .unwrap_err()
        .contains("User not found"));

    // Try to delete again (should fail with user not found)
    let double_delete_result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        global_admin,
        "delete_user_info",
        (charlie_principal_id,),
    )
    .expect("Failed to call delete_user_info for second time");

    assert!(double_delete_result.is_err());
    assert!(double_delete_result.unwrap_err().contains("User not found"));
}

#[test]
fn test_user_can_delete_their_own_info() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_service_canister = service_canisters.user_info_service_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    let charlie_principal_id = get_mock_user_charlie_principal_id();

    // Register the user
    let registration_result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        global_admin,
        "register_new_user",
        (charlie_principal_id,),
    )
    .expect("Failed to register new user");

    assert!(registration_result.is_ok());

    // User deletes their own info (should succeed)
    let self_delete_result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "delete_user_info",
        (charlie_principal_id,),
    )
    .expect("Failed to delete own user info");

    assert!(
        self_delete_result.is_ok(),
        "Self delete failed: {:?}",
        self_delete_result
    );

    // Verify user no longer exists
    let profile_after_delete = query::<_, Result<UserProfileDetailsForFrontendV3, String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "get_profile_details",
        (),
    )
    .expect("Failed to call get_profile_details");

    assert!(profile_after_delete.is_err());
    assert!(profile_after_delete.unwrap_err().contains("User not found"));
}
