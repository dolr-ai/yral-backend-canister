use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_charlie_principal_id,
};
use super::common::{RateLimitResult, register_user_for_testing};

#[test]
fn test_blacklist_property_blocks_requests() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let global_admin = get_global_super_admin_principal_id();
    let test_property = "test_property".to_string();
    
    // Register user in user_info_service
    register_user_for_testing(&pocket_ic, &service_canisters, charlie_principal_id)
        .expect("Failed to register user");
    
    // First, verify the user can normally make requests to the property
    // Use registered=true since unregistered users are rate limited by default (max_requests=0)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "check_rate_limit",
        (charlie_principal_id, test_property.clone(), true),
    )
    .expect("Failed to check rate limit");
    
    match result {
        RateLimitResult::Ok(_) => {}, // Expected - within rate limit
        RateLimitResult::Err(e) => panic!("Unexpected rate limit error before blacklist: {}", e),
    }
    
    // Add the property to blacklist
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "add_to_blacklist",
        (test_property.clone(),),
    )
    .expect("Failed to add property to blacklist");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("added to blacklist")),
        RateLimitResult::Err(e) => panic!("Failed to add to blacklist: {}", e),
    }
    
    // Verify the property is in the blacklist
    let blacklist = query::<_, Vec<String>>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_blacklist",
        (),
    )
    .expect("Failed to get blacklist");
    
    assert!(blacklist.contains(&test_property), "Property should be in blacklist");
    
    // Now check rate limit - should be blocked due to blacklist (rate limit = 0)
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "check_rate_limit",
        (charlie_principal_id, test_property.clone(), false),
    )
    .expect("Failed to check rate limit");
    
    match result {
        RateLimitResult::Ok(_) => panic!("Expected rate limit to be exceeded due to blacklist"),
        RateLimitResult::Err(msg) => assert!(msg.contains("Rate limit exceeded")),
    }
    
    // Remove from blacklist
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "remove_from_blacklist",
        (test_property.clone(),),
    )
    .expect("Failed to remove from blacklist");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("removed from blacklist")),
        RateLimitResult::Err(e) => panic!("Failed to remove from blacklist: {}", e),
    }
    
    // Verify the property is no longer blocked
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        charlie_principal_id,
        "check_rate_limit",
        (charlie_principal_id, test_property, true), // Use registered=true since unregistered users are always rate limited
    )
    .expect("Failed to check rate limit after removal");
    
    match result {
        RateLimitResult::Ok(_) => {}, // Expected - back to normal rate limiting
        RateLimitResult::Err(e) => panic!("Unexpected error after removing from blacklist: {}", e),
    }
}

#[test]
fn test_blacklist_all_blocks_all_properties() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    
    let rate_limits_canister = service_canisters.rate_limits_canister_id;
    let charlie_principal_id = get_mock_user_charlie_principal_id();
    let global_admin = get_global_super_admin_principal_id();
    
    // Register user in user_info_service
    register_user_for_testing(&pocket_ic, &service_canisters, charlie_principal_id)
        .expect("Failed to register user");
    
    // Test with different properties
    let properties = vec!["property1".to_string(), "property2".to_string(), "default".to_string()];
    
    // First, verify all properties work normally
    // Use registered=true since unregistered users are rate limited by default (max_requests=0)
    for property in &properties {
        let result = update::<_, RateLimitResult>(
            &pocket_ic,
            rate_limits_canister,
            charlie_principal_id,
            "check_rate_limit",
            (charlie_principal_id, property.clone(), true),
        )
        .expect("Failed to check rate limit");
        
        match result {
            RateLimitResult::Ok(_) => {}, // Expected
            RateLimitResult::Err(e) => panic!("Unexpected error for property {}: {}", property, e),
        }
    }
    
    // Add "all" to blacklist
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "add_to_blacklist",
        ("all".to_string(),),
    )
    .expect("Failed to add 'all' to blacklist");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("added to blacklist")),
        RateLimitResult::Err(e) => panic!("Failed to add 'all' to blacklist: {}", e),
    }
    
    // Now all properties should be blocked
    for property in &properties {
        let result = update::<_, RateLimitResult>(
            &pocket_ic,
            rate_limits_canister,
            charlie_principal_id,
            "check_rate_limit",
            (charlie_principal_id, property.clone(), false),
        )
        .expect("Failed to check rate limit");
        
        match result {
            RateLimitResult::Ok(_) => panic!("Expected {} to be blocked when 'all' is blacklisted", property),
            RateLimitResult::Err(msg) => assert!(msg.contains("Rate limit exceeded")),
        }
    }
    
    // Clear blacklist
    let result = update::<_, RateLimitResult>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "clear_blacklist",
        (),
    )
    .expect("Failed to clear blacklist");
    
    match result {
        RateLimitResult::Ok(msg) => assert!(msg.contains("cleared")),
        RateLimitResult::Err(e) => panic!("Failed to clear blacklist: {}", e),
    }
    
    // Verify blacklist is empty
    let blacklist = query::<_, Vec<String>>(
        &pocket_ic,
        rate_limits_canister,
        global_admin,
        "get_blacklist",
        (),
    )
    .expect("Failed to get blacklist");
    
    assert!(blacklist.is_empty(), "Blacklist should be empty after clearing");
    
    // All properties should work normally again
    // Use registered=true since unregistered users are always rate limited
    for property in &properties {
        let result = update::<_, RateLimitResult>(
            &pocket_ic,
            rate_limits_canister,
            charlie_principal_id,
            "check_rate_limit",
            (charlie_principal_id, property.clone(), true),
        )
        .expect("Failed to check rate limit after clearing");
        
        match result {
            RateLimitResult::Ok(_) => {}, // Expected
            RateLimitResult::Err(e) => panic!("Unexpected error for {} after clearing blacklist: {}", property, e),
        }
    }
}