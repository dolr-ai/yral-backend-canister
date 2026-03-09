use pocket_ic::PocketIc;
use shared_utils::canister_specific::individual_user_template::types::profile::{
    UserAccountType, UserProfileDetailsForFrontendV5, UserProfileDetailsForFrontendV7,
};
use shared_utils::canister_specific::user_info_service::types::{
    SubscriptionPlan, YralProSubscription,
};
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::get_new_pocket_ic_env_with_service_canisters_provisioned;
use test_utils::setup::test_constants::{
    get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
    get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id,
    get_mock_user_dan_principal_id,
};

fn setup_test_user(
    pocket_ic: &PocketIc,
    user_service_canister: candid::Principal,
    user_principal: candid::Principal,
) {
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

fn get_user_profile_v7(
    pocket_ic: &PocketIc,
    user_service_canister: candid::Principal,
    caller: candid::Principal,
    user_principal: candid::Principal,
) -> UserProfileDetailsForFrontendV7 {
    query::<_, Result<UserProfileDetailsForFrontendV7, String>>(
        pocket_ic,
        user_service_canister,
        caller,
        "get_user_profile_details_v7",
        (user_principal,),
    )
    .unwrap()
    .unwrap()
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

// ============================================================================
// Basic convert_to_bot_account tests
// ============================================================================

#[test]
fn test_convert_main_account_to_bot_account() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let owner_principal = get_mock_user_alice_principal_id();
    let bot_principal = get_mock_user_bob_principal_id();

    // Register both as regular users (MainAccount)
    setup_test_user(&pocket_ic, user_service_canister, owner_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot_principal);

    // Verify both are MainAccount before conversion
    let owner_profile = get_user_profile_v7(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        owner_principal,
    );
    assert!(
        matches!(owner_profile.account_type, UserAccountType::MainAccount { bots } if bots.is_empty())
    );

    let bot_profile_before = get_user_profile_v7(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        bot_principal,
    );
    assert!(
        matches!(bot_profile_before.account_type, UserAccountType::MainAccount { bots } if bots.is_empty())
    );

    // Convert bot to bot account under owner
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot_principal, owner_principal),
    )
    .expect("Failed to call convert_to_bot_account");
    assert!(result.is_ok(), "Conversion failed: {:?}", result);

    // Verify bot is now BotAccount
    let bot_profile_after = get_user_profile_v7(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        bot_principal,
    );
    match bot_profile_after.account_type {
        UserAccountType::BotAccount { owner } => {
            assert_eq!(owner, owner_principal);
        }
        _ => panic!(
            "Expected BotAccount, got: {:?}",
            bot_profile_after.account_type
        ),
    }

    // Verify owner's bots list contains the bot
    let owner_profile_after = get_user_profile_v7(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        owner_principal,
    );
    match owner_profile_after.account_type {
        UserAccountType::MainAccount { bots } => {
            assert_eq!(bots.len(), 1);
            assert!(bots.contains(&bot_principal));
        }
        _ => panic!(
            "Expected MainAccount, got: {:?}",
            owner_profile_after.account_type
        ),
    }
}

#[test]
fn test_convert_to_bot_account_fails_for_self() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let user_principal = get_mock_user_alice_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, user_principal);

    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (user_principal, user_principal),
    )
    .expect("Failed to call convert_to_bot_account");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot convert owner to its own bot");
}

#[test]
fn test_convert_to_bot_account_fails_when_owner_is_bot() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let owner_principal = get_mock_user_alice_principal_id();
    let bot1_principal = get_mock_user_bob_principal_id();
    let bot2_principal = get_mock_user_charlie_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, owner_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot1_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot2_principal);

    // Convert bot1 to bot under owner
    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot1_principal, owner_principal),
    )
    .expect("Failed to convert bot1");

    // Try to convert bot2 under bot1 (should fail)
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot2_principal, bot1_principal),
    )
    .expect("Failed to call convert_to_bot_account");

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Owner is a bot account, cannot own other bots"
    );
}

#[test]
fn test_convert_to_bot_account_fails_when_already_bot_of_owner() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let owner_principal = get_mock_user_alice_principal_id();
    let bot_principal = get_mock_user_bob_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, owner_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot_principal);

    // Convert once
    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot_principal, owner_principal),
    )
    .expect("Failed to convert bot");

    // Try to convert again (should fail)
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot_principal, owner_principal),
    )
    .expect("Failed to call convert_to_bot_account");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Already a bot of this owner");
}

#[test]
fn test_convert_to_bot_account_fails_for_non_existent_bot() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let owner_principal = get_mock_user_alice_principal_id();
    let non_existent = get_mock_user_dan_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, owner_principal);

    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (non_existent, owner_principal),
    )
    .expect("Failed to call convert_to_bot_account");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Bot principal not found");
}

#[test]
fn test_convert_to_bot_account_fails_for_non_existent_owner() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let bot_principal = get_mock_user_alice_principal_id();
    let non_existent = get_mock_user_dan_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, bot_principal);

    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot_principal, non_existent),
    )
    .expect("Failed to call convert_to_bot_account");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Owner not found");
}

#[test]
fn test_convert_multiple_bots_to_same_owner() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let owner_principal = get_mock_user_alice_principal_id();
    let bot1_principal = get_mock_user_bob_principal_id();
    let bot2_principal = get_mock_user_charlie_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, owner_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot1_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot2_principal);

    // Convert both bots
    let result1 = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot1_principal, owner_principal),
    )
    .expect("Failed to convert bot1");
    assert!(result1.is_ok());

    let result2 = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot2_principal, owner_principal),
    )
    .expect("Failed to convert bot2");
    assert!(result2.is_ok());

    // Verify owner has both bots
    let owner_profile = get_user_profile_v7(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        owner_principal,
    );
    match owner_profile.account_type {
        UserAccountType::MainAccount { bots } => {
            assert_eq!(bots.len(), 2);
            assert!(bots.contains(&bot1_principal));
            assert!(bots.contains(&bot2_principal));
        }
        _ => panic!("Expected MainAccount"),
    }
}

#[test]
fn test_convert_to_bot_account_rejects_non_admin_caller() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let owner_principal = get_mock_user_alice_principal_id();
    let bot_principal = get_mock_user_bob_principal_id();
    let non_admin = get_mock_user_charlie_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, owner_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot_principal);

    // Non-admin caller should be rejected (canister will trap)
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        non_admin,
        "convert_to_bot_account",
        (bot_principal, owner_principal),
    );

    assert!(result.is_err(), "Non-admin should not be able to call convert_to_bot_account");
}

// ============================================================================
// Pro subscription transfer tests
// ============================================================================

#[test]
fn test_convert_bot_with_pro_subscription_transfers_to_owner() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let owner_principal = get_mock_user_alice_principal_id();
    let bot_principal = get_mock_user_bob_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, owner_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot_principal);

    // Give bot a Pro subscription before conversion
    let pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 50,
        total_video_credits_alloted: 50,
    });

    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (bot_principal, pro_plan),
    )
    .expect("Failed to set bot Pro plan");

    // Verify bot has Pro and owner has Free before conversion
    let bot_plan_before = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        bot_principal,
        bot_principal,
    );
    assert!(matches!(bot_plan_before, SubscriptionPlan::Pro(_)));

    let owner_plan_before = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        owner_principal,
        owner_principal,
    );
    assert!(matches!(owner_plan_before, SubscriptionPlan::Free));

    // Convert bot to bot account
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot_principal, owner_principal),
    )
    .expect("Failed to convert bot");
    assert!(result.is_ok(), "Conversion failed: {:?}", result);

    // Verify Pro subscription was transferred to owner
    let owner_plan_after = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        owner_principal,
        owner_principal,
    );
    match owner_plan_after {
        SubscriptionPlan::Pro(pro_sub) => {
            assert_eq!(pro_sub.free_video_credits_left, 50);
            assert_eq!(pro_sub.total_video_credits_alloted, 50);
        }
        _ => panic!("Expected owner to have Pro plan after conversion"),
    }

    // Verify bot's own subscription was reset to Free
    // (bot now inherits from owner via get_effective_subscription_plan)
    let bot_plan_after = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        bot_principal,
        bot_principal,
    );
    // Bot should see the owner's Pro plan through inheritance
    match bot_plan_after {
        SubscriptionPlan::Pro(pro_sub) => {
            assert_eq!(pro_sub.free_video_credits_left, 50);
            assert_eq!(pro_sub.total_video_credits_alloted, 50);
        }
        _ => panic!("Expected bot to inherit Pro plan from owner"),
    }
}

#[test]
fn test_convert_bot_with_pro_does_not_overwrite_owner_pro() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let owner_principal = get_mock_user_alice_principal_id();
    let bot_principal = get_mock_user_bob_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, owner_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot_principal);

    // Give owner a Pro subscription
    let owner_pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 100,
        total_video_credits_alloted: 100,
    });
    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (owner_principal, owner_pro_plan),
    )
    .expect("Failed to set owner Pro plan");

    // Give bot a different Pro subscription
    let bot_pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 30,
        total_video_credits_alloted: 30,
    });
    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (bot_principal, bot_pro_plan),
    )
    .expect("Failed to set bot Pro plan");

    // Convert bot to bot account
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot_principal, owner_principal),
    )
    .expect("Failed to convert bot");
    assert!(result.is_ok());

    // Verify owner's Pro plan was NOT overwritten (kept the original 100 credits)
    let owner_plan_after = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        owner_principal,
        owner_principal,
    );
    match owner_plan_after {
        SubscriptionPlan::Pro(pro_sub) => {
            assert_eq!(pro_sub.free_video_credits_left, 100);
            assert_eq!(pro_sub.total_video_credits_alloted, 100);
        }
        _ => panic!("Expected owner to keep original Pro plan"),
    }
}

#[test]
fn test_convert_bot_with_free_plan_no_subscription_change() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let owner_principal = get_mock_user_alice_principal_id();
    let bot_principal = get_mock_user_bob_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, owner_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot_principal);

    // Both start with Free plan (default)
    let result = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot_principal, owner_principal),
    )
    .expect("Failed to convert bot");
    assert!(result.is_ok());

    // Verify both remain on Free plan
    let owner_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        owner_principal,
        owner_principal,
    );
    assert!(matches!(owner_plan, SubscriptionPlan::Free));

    let bot_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        bot_principal,
        bot_principal,
    );
    assert!(matches!(bot_plan, SubscriptionPlan::Free));
}

#[test]
fn test_converted_bot_inherits_owner_subscription_after_conversion() {
    let (pocket_ic, service_canisters) =
        get_new_pocket_ic_env_with_service_canisters_provisioned();
    let user_service_canister = service_canisters.user_info_service_canister_id;
    let admin_principal = get_global_super_admin_principal_id();
    let owner_principal = get_mock_user_alice_principal_id();
    let bot_principal = get_mock_user_bob_principal_id();

    setup_test_user(&pocket_ic, user_service_canister, owner_principal);
    setup_test_user(&pocket_ic, user_service_canister, bot_principal);

    // Convert bot first (both on Free plan)
    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "convert_to_bot_account",
        (bot_principal, owner_principal),
    )
    .expect("Failed to convert bot");

    // Now upgrade owner to Pro
    let pro_plan = SubscriptionPlan::Pro(YralProSubscription {
        free_video_credits_left: 75,
        total_video_credits_alloted: 75,
    });
    let _ = update::<_, Result<(), String>>(
        &pocket_ic,
        user_service_canister,
        admin_principal,
        "change_subscription_plan",
        (owner_principal, pro_plan),
    )
    .expect("Failed to upgrade owner");

    // Verify bot inherits owner's Pro plan
    let bot_plan = get_user_subscription_plan(
        &pocket_ic,
        user_service_canister,
        bot_principal,
        bot_principal,
    );
    match bot_plan {
        SubscriptionPlan::Pro(pro_sub) => {
            assert_eq!(pro_sub.free_video_credits_left, 75);
            assert_eq!(pro_sub.total_video_credits_alloted, 75);
        }
        _ => panic!("Expected bot to inherit owner's Pro plan"),
    }
}
