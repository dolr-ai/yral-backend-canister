use shared_utils::canister_specific::individual_user_template::types::{
    profile::UserProfileDetailsForFrontendV3,
    session::SessionType,
};
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_alice_principal_id, 
    get_mock_user_charlie_principal_id,
};

#[test]
fn test_accept_new_user_registration_for_user_info_service() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_service_canister = service_canisters.user_info_service_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let alice_principal_id = get_mock_user_alice_principal_id();
    let global_admin = get_global_super_admin_principal_id();

    // Test 1: Non-admin user should not be able to accept registrations
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "accept_new_user_registration",
        (charlie_principal_id, false),
    );
    assert!(result.is_err(), "Non-admin user registration should fail");

    // Test 2: Admin accepts user registration with authenticated=false (AnonymousSession)
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        global_admin,
        "accept_new_user_registration",
        (charlie_principal_id, false),
    );
    assert!(result.is_ok(), "User registration failed: {:?}", result);

    // Verify Charlie was registered with AnonymousSession
    let charlie_session_type = query::<_, Result<SessionType, String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "get_user_session_type",
        (charlie_principal_id,),
    )
    .unwrap()
    .unwrap();
    assert_eq!(charlie_session_type, SessionType::AnonymousSession);

    // Verify Charlie's profile exists
    let charlie_profile_details = query::<_, Result<UserProfileDetailsForFrontendV3, String>>(
        &pocket_ic,
        user_service_canister,
        charlie_principal_id,
        "get_user_profile_details",
        (charlie_principal_id,),
    )
    .unwrap()
    .unwrap();
    assert_eq!(charlie_profile_details.principal_id, charlie_principal_id);

    // Test 3: Admin accepts user registration with authenticated=true (RegisteredSession)
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        global_admin,
        "accept_new_user_registration",
        (alice_principal_id, true),
    );
    assert!(result.is_ok(), "Authenticated user registration failed: {:?}", result);

    // Verify Alice was registered with RegisteredSession
    let alice_session_type = query::<_, Result<SessionType, String>>(
        &pocket_ic,
        user_service_canister,
        alice_principal_id,
        "get_user_session_type",
        (alice_principal_id,),
    )
    .unwrap()
    .unwrap();
    assert_eq!(alice_session_type, SessionType::RegisteredSession);

    // Verify Alice's profile exists
    let alice_profile_details = query::<_, Result<UserProfileDetailsForFrontendV3, String>>(
        &pocket_ic,
        user_service_canister,
        alice_principal_id,
        "get_user_profile_details",
        (alice_principal_id,),
    )
    .unwrap()
    .unwrap();
    assert_eq!(alice_profile_details.principal_id, alice_principal_id);

    // Test 4: Try to register an already existing user (should fail)
    let duplicate_result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        global_admin,
        "accept_new_user_registration",
        (charlie_principal_id, false),
    )
    .unwrap();

    assert!(duplicate_result.is_err(), "Duplicate registration should fail");
    assert!(duplicate_result.unwrap_err().to_string().contains("User already exists"));
}
