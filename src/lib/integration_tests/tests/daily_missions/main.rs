use candid::{decode_one, encode_one, Principal};
use pocket_ic::{PocketIc, WasmResult};
use test_utils::setup::{
    env::pocket_ic_env::{
        get_new_pocket_ic_env_with_service_canisters_provisioned, ServiceCanisters,
    },
    test_constants::{
        get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
        get_mock_user_bob_principal_id, get_mock_user_charlie_principal_id,
    },
};

// Re-define types from the daily missions canister for testing
#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct UpdateMissionRequest {
    pub mission_type: MissionType,
    pub data: MissionUpdateData,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub enum MissionType {
    LoginStreak,
    PlayGames,
    GenerateAiVideos,
    Referrals,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub enum MissionUpdateData {
    LoginEvent,
    GamePlayed { game_id: String },
    AiVideoGenerated { video_id: String },
    ReferralMade { referred_user: Principal },
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct ClaimRewardRequest {
    pub reward_id: String,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct ClaimRewardResponse {
    pub success: bool,
    pub message: String,
    pub reward_amount: Option<u64>,
    pub claimed_reward: Option<ClaimedReward>,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct MissionUpdateResult {
    pub success: bool,
    pub message: String,
    pub new_progress: Option<MissionProgress>,
    pub reward_earned: Option<ClaimedReward>,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct MissionProgress {
    pub login_streak: LoginStreakProgress,
    pub game_progress: GameProgress,
    pub ai_video_progress: AiVideoProgress,
    pub referral_progress: ReferralProgress,
    pub pending_rewards: Vec<PendingReward>,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct LoginStreakProgress {
    pub current_day: u32,
    pub target_day: u32,
    pub can_claim: bool,
    pub reward_amount: u64,
    pub next_reset_in_hours: u32,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct GameProgress {
    pub current_count: u32,
    pub target_count: u32,
    pub can_claim: bool,
    pub reward_amount: u64,
    pub hours_remaining: u32,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct AiVideoProgress {
    pub current_count: u32,
    pub target_count: u32,
    pub can_claim: bool,
    pub reward_amount: u64,
    pub completed: bool,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct ReferralProgress {
    pub current_count: u32,
    pub target_count: u32,
    pub can_claim: bool,
    pub reward_amount: u64,
    pub completed: bool,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct PendingReward {
    pub id: String,
    pub reward_type: RewardType,
    pub amount: u64,
    pub earned_at: std::time::SystemTime,
    pub mission_day: u32,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum RewardType {
    LoginStreak,
    GameCompletion,
    AiVideoGeneration,
    Referral,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct ClaimedReward {
    pub reward_type: RewardType,
    pub amount: u64,
    pub claimed_at: std::time::SystemTime,
    pub day: u32,
}

// Re-define UserDailyMissions for testing
#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct UserDailyMissions {
    pub login_streak: LoginStreak,
    pub game_streak: GameStreak,
    pub ai_video_count: GenerateAiVideoCount,
    pub referral_count: ReferralCount,
    pub last_updated: std::time::SystemTime,
    pub pending_rewards: Vec<PendingReward>,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct LoginStreak {
    pub current_streak: u32,
    pub max_streak: u32,
    pub last_login_date: Option<std::time::SystemTime>,
    pub streak_start_date: Option<std::time::SystemTime>,
    pub claimed_rewards: Vec<ClaimedReward>,
    pub can_claim_today: bool,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct GameStreak {
    pub games_played_today: u32,
    pub target_games: u32,
    pub last_reset_date: Option<std::time::SystemTime>,
    pub claimed_today: bool,
    pub total_games_completed: u32,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct GenerateAiVideoCount {
    pub videos_generated_total: u32,
    pub target_videos: u32,
    pub completed: bool,
    pub reward_claimed: bool,
    pub total_videos_generated: u32,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct ReferralCount {
    pub referrals_made_total: u32,
    pub target_referrals: u32,
    pub completed: bool,
    pub reward_claimed: bool,
    pub total_referrals_made: u32,
    pub referred_users: Vec<ReferredUser>,
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct ReferredUser {
    pub principal_id: Principal,
    pub referred_at: std::time::SystemTime,
    pub is_active: bool,
}

mod helper {
    use super::*;

    pub fn get_mission_progress(
        pic: &PocketIc,
        canister_id: Principal,
        sender: Principal,
    ) -> MissionProgress {
        let res = pic
            .query_call(
                canister_id,
                sender,
                "get_current_mission_progress",
                encode_one(()).unwrap(),
            )
            .unwrap();

        match res {
            WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
            WasmResult::Reject(reason) => panic!("Query call rejected: {reason}"),
        }
    }

    pub fn get_user_missions(
        pic: &PocketIc,
        canister_id: Principal,
        sender: Principal,
    ) -> UserDailyMissions {
        let res = pic
            .query_call(
                canister_id,
                sender,
                "get_current_missions",
                encode_one(()).unwrap(),
            )
            .unwrap();

        match res {
            WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
            WasmResult::Reject(reason) => panic!("Query call rejected: {reason}"),
        }
    }

    pub fn update_mission(
        pic: &PocketIc,
        canister_id: Principal,
        sender: Principal,
        request: UpdateMissionRequest,
    ) -> MissionUpdateResult {
        let res = pic
            .update_call(
                canister_id,
                sender,
                "update_mission",
                encode_one(request).unwrap(),
            )
            .unwrap();

        match res {
            WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
            WasmResult::Reject(reason) => panic!("Update call rejected: {reason}"),
        }
    }

    pub fn claim_reward(
        pic: &PocketIc,
        canister_id: Principal,
        sender: Principal,
        reward_id: String,
    ) -> ClaimRewardResponse {
        let res = pic
            .update_call(
                canister_id,
                sender,
                "claim_reward",
                encode_one(ClaimRewardRequest { reward_id }).unwrap(),
            )
            .unwrap();

        match res {
            WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
            WasmResult::Reject(reason) => panic!("Claim reward call rejected: {reason}"),
        }
    }

    pub fn get_version(pic: &PocketIc, canister_id: Principal, sender: Principal) -> String {
        let res = pic
            .query_call(canister_id, sender, "get_version", encode_one(()).unwrap())
            .unwrap();

        match res {
            WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
            WasmResult::Reject(reason) => panic!("Get version call rejected: {reason}"),
        }
    }

    pub fn get_user_missions_for_principal(
        pic: &PocketIc,
        canister_id: Principal,
        sender: Principal,
        target_principal: Principal,
    ) -> UserDailyMissions {
        let res = pic
            .query_call(
                canister_id,
                sender,
                "get_user_missions_for_principal",
                encode_one(target_principal).unwrap(),
            )
            .unwrap();

        match res {
            WasmResult::Reply(bytes) => decode_one(&bytes).unwrap(),
            WasmResult::Reject(reason) => panic!("Query user missions call rejected: {reason}"),
        }
    }
}

#[test]
fn test_daily_missions_canister_version() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let sender = get_global_super_admin_principal_id();
    let version = helper::get_version(&pic, canister_id, sender);

    assert_eq!(version, "v1.0.0");
}

#[test]
fn test_initial_mission_progress() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();
    let progress = helper::get_mission_progress(&pic, canister_id, alice);

    // Initial progress should have default values
    assert_eq!(progress.login_streak.current_day, 0);
    assert_eq!(progress.game_progress.current_count, 0);
    assert_eq!(progress.ai_video_progress.current_count, 0);
    assert_eq!(progress.referral_progress.current_count, 0);
    assert!(progress.pending_rewards.is_empty());
}

#[test]
fn test_login_streak_mission_progression() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();

    // Update login mission
    let login_request = UpdateMissionRequest {
        mission_type: MissionType::LoginStreak,
        data: MissionUpdateData::LoginEvent,
    };

    let result = helper::update_mission(&pic, canister_id, alice, login_request);
    assert!(result.success);
    assert!(result.message.contains("Login streak updated"));

    // Check progress after login
    let progress = helper::get_mission_progress(&pic, canister_id, alice);
    assert_eq!(progress.login_streak.current_day, 1);

    // Login multiple times to reach target
    for day in 2..=7 {
        let login_request = UpdateMissionRequest {
            mission_type: MissionType::LoginStreak,
            data: MissionUpdateData::LoginEvent,
        };

        let result = helper::update_mission(&pic, canister_id, alice, login_request);
        assert!(result.success);

        let progress = helper::get_mission_progress(&pic, canister_id, alice);
        assert_eq!(progress.login_streak.current_day, day);

        if day >= progress.login_streak.target_day {
            assert!(progress.login_streak.can_claim || !progress.pending_rewards.is_empty());
        }
    }
}

#[test]
fn test_game_mission_progression() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();

    // Play games to meet target
    for game_num in 1..=5 {
        let game_request = UpdateMissionRequest {
            mission_type: MissionType::PlayGames,
            data: MissionUpdateData::GamePlayed {
                game_id: format!("game_{}", game_num),
            },
        };

        let result = helper::update_mission(&pic, canister_id, alice, game_request);
        assert!(result.success);

        let progress = helper::get_mission_progress(&pic, canister_id, alice);
        assert_eq!(progress.game_progress.current_count, game_num);
    }

    let final_progress = helper::get_mission_progress(&pic, canister_id, alice);

    // Should have completed the target and have a pending reward or be able to claim
    if final_progress.game_progress.current_count >= final_progress.game_progress.target_count {
        assert!(
            final_progress.game_progress.can_claim || !final_progress.pending_rewards.is_empty()
        );
    }
}

#[test]
fn test_ai_video_generation_mission() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();
    let initial_progress = helper::get_mission_progress(&pic, canister_id, alice);
    let target_videos = initial_progress.ai_video_progress.target_count;

    // Generate AI videos to meet target
    for video_num in 1..=target_videos {
        let video_request = UpdateMissionRequest {
            mission_type: MissionType::GenerateAiVideos,
            data: MissionUpdateData::AiVideoGenerated {
                video_id: format!("video_{}", video_num),
            },
        };

        let result = helper::update_mission(&pic, canister_id, alice, video_request);
        assert!(result.success);

        let progress = helper::get_mission_progress(&pic, canister_id, alice);
        assert_eq!(progress.ai_video_progress.current_count, video_num);
    }

    let final_progress = helper::get_mission_progress(&pic, canister_id, alice);
    assert!(final_progress.ai_video_progress.completed);
    assert!(
        final_progress.ai_video_progress.can_claim || !final_progress.pending_rewards.is_empty()
    );
}

#[test]
fn test_referral_mission_progression() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();
    let bob = get_mock_user_bob_principal_id();
    let charlie = get_mock_user_charlie_principal_id();

    let initial_progress = helper::get_mission_progress(&pic, canister_id, alice);
    let target_referrals = initial_progress.referral_progress.target_count;

    let referred_users = vec![bob, charlie];

    // Make referrals up to target
    for (index, &referred_user) in referred_users.iter().enumerate() {
        if (index + 1) as u32 > target_referrals {
            break;
        }

        let referral_request = UpdateMissionRequest {
            mission_type: MissionType::Referrals,
            data: MissionUpdateData::ReferralMade { referred_user },
        };

        let result = helper::update_mission(&pic, canister_id, alice, referral_request);
        assert!(result.success);

        let progress = helper::get_mission_progress(&pic, canister_id, alice);
        assert_eq!(progress.referral_progress.current_count, (index + 1) as u32);
    }

    let final_progress = helper::get_mission_progress(&pic, canister_id, alice);
    if final_progress.referral_progress.current_count >= target_referrals {
        assert!(final_progress.referral_progress.completed);
        assert!(
            final_progress.referral_progress.can_claim
                || !final_progress.pending_rewards.is_empty()
        );
    }
}

#[test]
fn test_reward_claiming_flow() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();

    // Complete a mission to generate a reward
    let initial_progress = helper::get_mission_progress(&pic, canister_id, alice);
    let target_videos = initial_progress.ai_video_progress.target_count;

    for video_num in 1..=target_videos {
        let video_request = UpdateMissionRequest {
            mission_type: MissionType::GenerateAiVideos,
            data: MissionUpdateData::AiVideoGenerated {
                video_id: format!("video_{}", video_num),
            },
        };
        helper::update_mission(&pic, canister_id, alice, video_request);
    }

    let progress_after_completion = helper::get_mission_progress(&pic, canister_id, alice);

    // Should have pending rewards
    assert!(!progress_after_completion.pending_rewards.is_empty());

    let pending_reward = &progress_after_completion.pending_rewards[0];
    let reward_id = pending_reward.id.clone();
    let expected_amount = pending_reward.amount;

    // Claim the reward
    let claim_response = helper::claim_reward(&pic, canister_id, alice, reward_id);

    assert!(claim_response.success);
    assert_eq!(claim_response.reward_amount, Some(expected_amount));
    assert!(claim_response.claimed_reward.is_some());

    // Verify reward is no longer pending
    let progress_after_claim = helper::get_mission_progress(&pic, canister_id, alice);
    assert!(progress_after_claim.pending_rewards.is_empty());
}

#[test]
fn test_invalid_reward_claim() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();

    // Try to claim a non-existent reward
    let claim_response =
        helper::claim_reward(&pic, canister_id, alice, "invalid_reward_id".to_string());

    assert!(!claim_response.success);
    assert!(
        claim_response.message.contains("not found")
            || claim_response.message.contains("already claimed")
    );
    assert_eq!(claim_response.reward_amount, None);
    assert_eq!(claim_response.claimed_reward, None);
}

#[test]
fn test_invalid_mission_update_data() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();

    // Try to send game data for login streak mission (should fail)
    let invalid_request = UpdateMissionRequest {
        mission_type: MissionType::LoginStreak,
        data: MissionUpdateData::GamePlayed {
            game_id: "game_123".to_string(),
        },
    };

    let result = helper::update_mission(&pic, canister_id, alice, invalid_request);
    assert!(!result.success);
    assert!(result.message.contains("Invalid data"));
}

#[test]
fn test_multiple_users_independence() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();
    let bob = get_mock_user_bob_principal_id();

    // Alice completes some missions
    let alice_login_request = UpdateMissionRequest {
        mission_type: MissionType::LoginStreak,
        data: MissionUpdateData::LoginEvent,
    };
    helper::update_mission(&pic, canister_id, alice, alice_login_request);

    // Bob completes different missions
    let bob_game_request = UpdateMissionRequest {
        mission_type: MissionType::PlayGames,
        data: MissionUpdateData::GamePlayed {
            game_id: "bob_game_1".to_string(),
        },
    };
    helper::update_mission(&pic, canister_id, bob, bob_game_request);

    // Check that their progress is independent
    let alice_progress = helper::get_mission_progress(&pic, canister_id, alice);
    let bob_progress = helper::get_mission_progress(&pic, canister_id, bob);

    assert_eq!(alice_progress.login_streak.current_day, 1);
    assert_eq!(alice_progress.game_progress.current_count, 0);

    assert_eq!(bob_progress.login_streak.current_day, 0);
    assert_eq!(bob_progress.game_progress.current_count, 1);
}

#[test]
fn test_mission_progress_persistence() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();

    // Complete multiple missions
    let login_request = UpdateMissionRequest {
        mission_type: MissionType::LoginStreak,
        data: MissionUpdateData::LoginEvent,
    };
    helper::update_mission(&pic, canister_id, alice, login_request);

    let game_request = UpdateMissionRequest {
        mission_type: MissionType::PlayGames,
        data: MissionUpdateData::GamePlayed {
            game_id: "persistent_game".to_string(),
        },
    };
    helper::update_mission(&pic, canister_id, alice, game_request);

    // Check progress persists across multiple queries
    let progress1 = helper::get_mission_progress(&pic, canister_id, alice);
    let progress2 = helper::get_mission_progress(&pic, canister_id, alice);

    assert_eq!(
        progress1.login_streak.current_day,
        progress2.login_streak.current_day
    );
    assert_eq!(
        progress1.game_progress.current_count,
        progress2.game_progress.current_count
    );
    assert_eq!(progress1.login_streak.current_day, 1);
    assert_eq!(progress1.game_progress.current_count, 1);
}

#[test]
fn test_cross_mission_completion() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();
    let bob = get_mock_user_bob_principal_id();

    // Alice completes all mission types in sequence
    let login_request = UpdateMissionRequest {
        mission_type: MissionType::LoginStreak,
        data: MissionUpdateData::LoginEvent,
    };
    let game_request = UpdateMissionRequest {
        mission_type: MissionType::PlayGames,
        data: MissionUpdateData::GamePlayed {
            game_id: "cross_mission_game".to_string(),
        },
    };
    let video_request = UpdateMissionRequest {
        mission_type: MissionType::GenerateAiVideos,
        data: MissionUpdateData::AiVideoGenerated {
            video_id: "cross_mission_video".to_string(),
        },
    };
    let referral_request = UpdateMissionRequest {
        mission_type: MissionType::Referrals,
        data: MissionUpdateData::ReferralMade { referred_user: bob },
    };

    // Execute all mission types
    let login_result = helper::update_mission(&pic, canister_id, alice, login_request);
    let game_result = helper::update_mission(&pic, canister_id, alice, game_request);
    let video_result = helper::update_mission(&pic, canister_id, alice, video_request);
    let referral_result = helper::update_mission(&pic, canister_id, alice, referral_request);

    // All should succeed
    assert!(login_result.success);
    assert!(game_result.success);
    assert!(video_result.success);
    assert!(referral_result.success);

    // Final progress should reflect all completions
    let final_progress = helper::get_mission_progress(&pic, canister_id, alice);
    assert_eq!(final_progress.login_streak.current_day, 1);
    assert_eq!(final_progress.game_progress.current_count, 1);
    assert_eq!(final_progress.ai_video_progress.current_count, 1);
    assert_eq!(final_progress.referral_progress.current_count, 1);
}

#[test]
fn test_admin_can_query_any_user_missions() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let admin = get_global_super_admin_principal_id();
    let alice = get_mock_user_alice_principal_id();

    // Alice completes some missions
    let alice_login_request = UpdateMissionRequest {
        mission_type: MissionType::LoginStreak,
        data: MissionUpdateData::LoginEvent,
    };
    helper::update_mission(&pic, canister_id, alice, alice_login_request);

    // Admin queries Alice's missions
    let alice_missions = helper::get_user_missions_for_principal(&pic, canister_id, admin, alice);

    assert_eq!(alice_missions.login_streak.current_streak, 1);
    assert!(alice_missions.login_streak.last_login_date.is_some());
}

#[test]
fn test_double_claim_prevention() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();

    // Complete AI video mission to generate reward
    let initial_progress = helper::get_mission_progress(&pic, canister_id, alice);
    let target_videos = initial_progress.ai_video_progress.target_count;

    for video_num in 1..=target_videos {
        let video_request = UpdateMissionRequest {
            mission_type: MissionType::GenerateAiVideos,
            data: MissionUpdateData::AiVideoGenerated {
                video_id: format!("video_{}", video_num),
            },
        };
        helper::update_mission(&pic, canister_id, alice, video_request);
    }

    let progress_with_reward = helper::get_mission_progress(&pic, canister_id, alice);
    assert!(!progress_with_reward.pending_rewards.is_empty());

    let reward_id = progress_with_reward.pending_rewards[0].id.clone();

    // Claim the reward successfully
    let first_claim = helper::claim_reward(&pic, canister_id, alice, reward_id.clone());
    assert!(first_claim.success);
    assert!(first_claim.reward_amount.is_some());

    // Try to claim the same reward again - should fail
    let second_claim = helper::claim_reward(&pic, canister_id, alice, reward_id);
    assert!(!second_claim.success);
    assert!(
        second_claim.message.contains("not found")
            || second_claim.message.contains("already claimed")
    );
    assert_eq!(second_claim.reward_amount, None);
}

#[test]
fn test_mission_reward_amounts() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();

    // Test AI video generation reward
    let initial_progress = helper::get_mission_progress(&pic, canister_id, alice);
    let target_videos = initial_progress.ai_video_progress.target_count;
    let expected_video_reward = initial_progress.ai_video_progress.reward_amount;

    for video_num in 1..=target_videos {
        let video_request = UpdateMissionRequest {
            mission_type: MissionType::GenerateAiVideos,
            data: MissionUpdateData::AiVideoGenerated {
                video_id: format!("reward_test_video_{}", video_num),
            },
        };
        helper::update_mission(&pic, canister_id, alice, video_request);
    }

    let progress_after_videos = helper::get_mission_progress(&pic, canister_id, alice);
    assert!(!progress_after_videos.pending_rewards.is_empty());

    let pending_reward = &progress_after_videos.pending_rewards[0];
    assert_eq!(pending_reward.amount, expected_video_reward);

    // Claim and verify the amount
    let claim_response = helper::claim_reward(&pic, canister_id, alice, pending_reward.id.clone());
    assert!(claim_response.success);
    assert_eq!(claim_response.reward_amount, Some(expected_video_reward));
}

#[test]
fn test_referral_with_multiple_users() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();
    let bob = get_mock_user_bob_principal_id();
    let charlie = get_mock_user_charlie_principal_id();

    // Alice refers both Bob and Charlie
    let referral_bob = UpdateMissionRequest {
        mission_type: MissionType::Referrals,
        data: MissionUpdateData::ReferralMade { referred_user: bob },
    };

    let referral_charlie = UpdateMissionRequest {
        mission_type: MissionType::Referrals,
        data: MissionUpdateData::ReferralMade {
            referred_user: charlie,
        },
    };

    helper::update_mission(&pic, canister_id, alice, referral_bob);
    helper::update_mission(&pic, canister_id, alice, referral_charlie);

    let alice_progress = helper::get_mission_progress(&pic, canister_id, alice);
    assert_eq!(alice_progress.referral_progress.current_count, 2);

    // Verify that Bob and Charlie still have clean states
    let bob_progress = helper::get_mission_progress(&pic, canister_id, bob);
    let charlie_progress = helper::get_mission_progress(&pic, canister_id, charlie);

    assert_eq!(bob_progress.referral_progress.current_count, 0);
    assert_eq!(charlie_progress.referral_progress.current_count, 0);
}

#[test]
fn test_mission_target_values() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();
    let progress = helper::get_mission_progress(&pic, canister_id, alice);

    // Verify target values match expected defaults
    assert_eq!(progress.login_streak.target_day, 7); // LOGIN_STREAK_TARGET
    assert_eq!(progress.game_progress.target_count, 10); // GAME_TARGET
    assert_eq!(progress.ai_video_progress.target_count, 3); // AI_VIDEO_TARGET
    assert_eq!(progress.referral_progress.target_count, 3); // REFERRAL_TARGET

    // Verify reward amounts are as expected (these should match constants from the canister)
    assert!(progress.login_streak.reward_amount > 0);
    assert!(progress.game_progress.reward_amount > 0);
    assert!(progress.ai_video_progress.reward_amount > 0);
    assert!(progress.referral_progress.reward_amount > 0);
}

#[test]
fn test_mission_state_after_exceeding_targets() {
    let (
        pic,
        ServiceCanisters {
            daily_missions_canister_id: canister_id,
            ..
        },
    ) = get_new_pocket_ic_env_with_service_canisters_provisioned();

    let alice = get_mock_user_alice_principal_id();
    let initial_progress = helper::get_mission_progress(&pic, canister_id, alice);
    let target_videos = initial_progress.ai_video_progress.target_count;

    // Generate more videos than the target
    for video_num in 1..=(target_videos + 2) {
        let video_request = UpdateMissionRequest {
            mission_type: MissionType::GenerateAiVideos,
            data: MissionUpdateData::AiVideoGenerated {
                video_id: format!("exceed_target_video_{}", video_num),
            },
        };
        helper::update_mission(&pic, canister_id, alice, video_request);
    }

    let final_progress = helper::get_mission_progress(&pic, canister_id, alice);

    // Should be completed and have current count exceeding target
    assert!(final_progress.ai_video_progress.completed);
    assert_eq!(
        final_progress.ai_video_progress.current_count,
        target_videos + 2
    );

    // Should still be able to claim reward or have pending reward
    assert!(
        final_progress.ai_video_progress.can_claim || !final_progress.pending_rewards.is_empty()
    );
}
