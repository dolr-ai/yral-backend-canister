use test_utils::canister_calls::query;
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::get_mock_user_charlie_principal_id;

#[test]
fn test_get_version() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    
    let version = query::<_, String>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "get_version",
        (),
    )
    .expect("Failed to get version");
    
    // Version should not be empty
    assert!(!version.is_empty(), "Version should not be empty");
    
    // The version format is "v1.0.0" not "1.0.0", so we need to handle the 'v' prefix
    let version_without_prefix = version.strip_prefix('v').unwrap_or(&version);
    
    // Version should follow semantic versioning format (e.g., "1.0.0")
    let parts: Vec<&str> = version_without_prefix.split('.').collect();
    assert!(parts.len() >= 2, "Version should have at least major.minor format");
    
    // Each part should be a valid number
    for part in parts {
        assert!(part.parse::<u32>().is_ok(), "Version part '{}' is not a valid number", part);
    }
}