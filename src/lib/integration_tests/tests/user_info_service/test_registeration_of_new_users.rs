use pocket_ic::PocketIc;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontendV3;
use shared_utils::constant::get_global_super_admin_principal_id_v1;
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};

#[test]
fn test_registration_of_new_users() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let user_service_canister = service_canisters.user_info_service_canister_id;
    let global_admin = get_global_super_admin_principal_id();
    let charlie_principal_id = get_mock_user_charlie_principal_id();

    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        global_admin,
        "register_new_user",
        (charlie_principal_id,),
    )
    .expect("Failed to register new user");

    assert!(result.is_ok(), "User registration failed: {:?}", result);

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
}
