use pocket_ic::PocketIc;
use shared_utils::canister_specific::individual_user_template::types::profile::UserProfileDetailsForFrontendV5;
use shared_utils::canister_specific::user_info_service::types::{
    SubscriptionPlan, YralProSubscription,
};
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
    get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id,
};

fn setup_test_user(
    pocket_ic: &PocketIc,
    user_service_canister: candid::Principal,
    user_principal: candid::Principal,
) {
    // Register the user first
    let result = update::<_, Result<(), String>>(
        pocket_ic,
        user_service_canister,
        user_principal,
        "register_new_user",
        (),
    )
    .expect("Failed to register new user");
    assert!(result.is_ok(), "User registration failed: {:?}", result);
}

fn get_user_subscription_plan(
    pocket_ic: &PocketIc,
    user_service_canister: candid::Principal,
    caller: candid::Principal,
    user_principal: candid::Principal,
) -> SubscriptionPlan {
    let profile_details = query::<_, Result<UserProfileDetailsForFrontendV5, String>>(
        pocket_ic,
        user_service_canister,
        caller,
        "get_user_profile_details_v5",
        (user_principal,),
    )
    .unwrap()
    .unwrap();

    profile_details.subscription_plan
}

#[test]
fn test_change_subscription_plan_from_free_to_pro() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let user_principal = get_mock_user_alice_principal_id();

    // Setup user with default Free plan
    setup_test_user(&pocket_ic, user_service_canister, user_principal);

    // Verify user starts with Free plan
    let initial_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        user_principal,
        user_principal,
    );
    assert!(matches!(initial_plan, SubscriptionPlan::Free));

    // Change to Pro plan
    let new_pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 50,
    });

    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (user_principal, new_pro_plan.clone()),
    )
    .expect("Failed to change subscription plan");

    assert!(
        result.is_ok(),
        "Subscription plan change failed: {:?}",
        result
    );

    // Verify the plan was changed
    let updated_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        user_principal,
        user_principal,
    );
    match updated_plan {
        SubscriptionPlan::Pro(pro_sub) => {
            assert_eq!(pro_sub.free_video_credits_left, 50);
        }
        _ => panic!("Expected Pro plan, got: {:?}", updated_plan),
    }
}

#[test]
fn test_change_subscription_plan_from_pro_to_free() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let user_principal = get_mock_user_bob_principal_id();

    // Setup user and change to Pro plan first
    setup_test_user(&pocket_ic, user_service_canister, user_principal);

    let pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 30,
    });

    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (user_principal, pro_plan),
    )
    .expect("Failed to set initial Pro plan");

    // Now change back to Free plan
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (user_principal, SubscriptionPlan::Free),
    )
    .expect("Failed to change subscription plan");

    assert!(
        result.is_ok(),
        "Subscription plan change failed: {:?}",
        result
    );

    // Verify the plan was changed to Free
    let updated_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        user_principal,
        user_principal,
    );
    assert!(matches!(updated_plan, SubscriptionPlan::Free));
}

#[test]
fn test_change_subscription_plan_for_non_existent_user() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let non_existent_user = get_mock_user_charlie_principal_id(); // Don't register this user

    let new_pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 50,
    });

    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (non_existent_user, new_pro_plan),
    )
    .expect("Failed to call change_subscription_plan");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User not found");
}

#[test]
fn test_add_pro_plan_free_video_credits_success() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let user_principal = get_mock_user_alice_principal_id();

    // Setup user and change to Pro plan
    setup_test_user(&pocket_ic, user_service_canister, user_principal);

    let pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 10,
    });

    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (user_principal, pro_plan),
    )
    .expect("Failed to set Pro plan");

    // Add 20 credits
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "add_pro_plan_free_video_credits",
        (user_principal, 20u32),
    )
    .expect("Failed to add video credits");

    assert!(result.is_ok(), "Adding video credits failed: {:?}", result);

    // Verify credits were added (10 + 20 = 30)
    let updated_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        user_principal,
        user_principal,
    );
    match updated_plan {
        SubscriptionPlan::Pro(pro_sub) => {
            assert_eq!(pro_sub.free_video_credits_left, 30);
        }
        _ => panic!("Expected Pro plan, got: {:?}", updated_plan),
    }
}

#[test]
fn test_add_pro_plan_free_video_credits_for_free_user() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let user_principal = get_mock_user_bob_principal_id();

    // Setup user with Free plan (default)
    setup_test_user(&pocket_ic, user_service_canister, user_principal);

    // Try to add credits to Free plan user
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "add_pro_plan_free_video_credits",
        (user_principal, 10u32),
    )
    .expect("Failed to call add_pro_plan_free_video_credits");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User is on Free plan");
}

#[test]
fn test_add_pro_plan_free_video_credits_for_non_existent_user() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let non_existent_user = get_mock_user_charlie_principal_id(); // Don't register this user

    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "add_pro_plan_free_video_credits",
        (non_existent_user, 10u32),
    )
    .expect("Failed to call add_pro_plan_free_video_credits");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User not found");
}

#[test]
fn test_remove_pro_plan_free_video_credits_success() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let user_principal = get_mock_user_alice_principal_id();

    // Setup user and change to Pro plan with credits
    setup_test_user(&pocket_ic, user_service_canister, user_principal);

    let pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 50,
    });

    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (user_principal, pro_plan),
    )
    .expect("Failed to set Pro plan");

    // Remove 20 credits
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "remove_pro_plan_free_video_credits",
        (user_principal, 20u32),
    )
    .expect("Failed to remove video credits");

    assert!(
        result.is_ok(),
        "Removing video credits failed: {:?}",
        result
    );

    // Verify credits were removed (50 - 20 = 30)
    let updated_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        user_principal,
        user_principal,
    );
    match updated_plan {
        SubscriptionPlan::Pro(pro_sub) => {
            assert_eq!(pro_sub.free_video_credits_left, 30);
        }
        _ => panic!("Expected Pro plan, got: {:?}", updated_plan),
    }
}

#[test]
fn test_remove_pro_plan_free_video_credits_insufficient_credits() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let user_principal = get_mock_user_bob_principal_id();

    // Setup user and change to Pro plan with limited credits
    setup_test_user(&pocket_ic, user_service_canister, user_principal);

    let pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 5,
    });

    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (user_principal, pro_plan),
    )
    .expect("Failed to set Pro plan");

    // Try to remove more credits than available
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "remove_pro_plan_free_video_credits",
        (user_principal, 10u32),
    )
    .expect("Failed to call remove_pro_plan_free_video_credits");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Not enough free video credits");

    // Verify credits remain unchanged
    let updated_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        user_principal,
        user_principal,
    );
    match updated_plan {
        SubscriptionPlan::Pro(pro_sub) => {
            assert_eq!(pro_sub.free_video_credits_left, 5);
        }
        _ => panic!("Expected Pro plan, got: {:?}", updated_plan),
    }
}

#[test]
fn test_remove_pro_plan_free_video_credits_for_free_user() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let user_principal = get_mock_user_charlie_principal_id();

    // Setup user with Free plan (default)
    setup_test_user(&pocket_ic, user_service_canister, user_principal);

    // Try to remove credits from Free plan user
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "remove_pro_plan_free_video_credits",
        (user_principal, 5u32),
    )
    .expect("Failed to call remove_pro_plan_free_video_credits");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User is on Free plan");
}

#[test]
fn test_remove_pro_plan_free_video_credits_for_non_existent_user() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let non_existent_user = candid::Principal::anonymous(); // Use a different non-existent user

    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "remove_pro_plan_free_video_credits",
        (non_existent_user, 5u32),
    )
    .expect("Failed to call remove_pro_plan_free_video_credits");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "User not found");
}

#[test]
fn test_remove_all_pro_plan_free_video_credits() {
    let (pocket_ic, service_canisters) = get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let user_principal = get_mock_user_alice_principal_id();

    // Setup user and change to Pro plan with credits
    setup_test_user(&pocket_ic, user_service_canister, user_principal);

    let pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 25,
    });

    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (user_principal, pro_plan),
    )
    .expect("Failed to set Pro plan");

    // Remove exactly all credits
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "remove_pro_plan_free_video_credits",
        (user_principal, 25u32),
    )
    .expect("Failed to remove video credits");

    assert!(
        result.is_ok(),
        "Removing all video credits failed: {:?}",
        result
    );

    // Verify all credits were removed
    let updated_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        user_principal,
        user_principal,
    );
    match updated_plan {
        SubscriptionPlan::Pro(pro_sub) => {
            assert_eq!(pro_sub.free_video_credits_left, 0);
        }
        _ => panic!("Expected Pro plan, got: {:?}", updated_plan),
    }
}
