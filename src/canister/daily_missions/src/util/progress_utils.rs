use super::mission_utils::{calculate_hours_until_next_day, calculate_hours_until_reset};
use crate::data_model::{
    AiVideoProgress, GameProgress, LoginStreakProgress, MissionProgress, PendingReward,
    ReferralProgress, RewardType, UserDailyMissions, AI_VIDEO_GENERATION_REWARD,
    GAME_COMPLETION_REWARD, LOGIN_STREAK_REWARD, LOGIN_STREAK_TARGET, REFERRAL_REWARD,
};
use std::time::SystemTime;

/// Builds comprehensive mission progress for a user
pub fn build_mission_progress(missions: &UserDailyMissions, now: SystemTime) -> MissionProgress {
    let has_pending_login_reward = missions
        .pending_rewards
        .iter()
        .any(|r| r.reward_type == RewardType::LoginStreak);

    let has_pending_game_reward = missions
        .pending_rewards
        .iter()
        .any(|r| r.reward_type == RewardType::GameCompletion);

    let has_pending_ai_reward = missions
        .pending_rewards
        .iter()
        .any(|r| r.reward_type == RewardType::AiVideoGeneration);

    let has_pending_referral_reward = missions
        .pending_rewards
        .iter()
        .any(|r| r.reward_type == RewardType::Referral);

    MissionProgress {
        login_streak: LoginStreakProgress {
            current_day: missions.login_streak.current_streak,
            target_day: LOGIN_STREAK_TARGET,
            can_claim: has_pending_login_reward
                || (missions.login_streak.can_claim_today
                    && missions.login_streak.current_streak >= LOGIN_STREAK_TARGET),
            reward_amount: LOGIN_STREAK_REWARD,
            next_reset_in_hours: calculate_hours_until_next_day(now),
        },
        game_progress: GameProgress {
            current_count: missions.game_streak.games_played_today,
            target_count: missions.game_streak.target_games,
            can_claim: has_pending_game_reward
                || (!missions.game_streak.claimed_today
                    && missions.game_streak.games_played_today
                        >= missions.game_streak.target_games),
            reward_amount: GAME_COMPLETION_REWARD,
            hours_remaining: calculate_hours_until_reset(
                &missions.game_streak.last_reset_date,
                now,
            ),
        },
        ai_video_progress: AiVideoProgress {
            current_count: missions.ai_video_count.videos_generated_total,
            target_count: missions.ai_video_count.target_videos,
            can_claim: has_pending_ai_reward
                || (!missions.ai_video_count.reward_claimed && missions.ai_video_count.completed),
            reward_amount: AI_VIDEO_GENERATION_REWARD,
            completed: missions.ai_video_count.completed,
        },
        referral_progress: ReferralProgress {
            current_count: missions.referral_count.referrals_made_total,
            target_count: missions.referral_count.target_referrals,
            can_claim: has_pending_referral_reward
                || (!missions.referral_count.reward_claimed && missions.referral_count.completed),
            reward_amount: REFERRAL_REWARD,
            completed: missions.referral_count.completed,
        },
        pending_rewards: missions.pending_rewards.clone(),
    }
}

/// Calculates total pending reward amount for a user
pub fn calculate_total_pending_rewards(pending_rewards: &[PendingReward]) -> u64 {
    pending_rewards.iter().map(|r| r.amount).sum()
}

/// Finds a pending reward by ID
pub fn find_pending_reward_by_id<'a>(
    pending_rewards: &'a [PendingReward],
    reward_id: &str,
) -> Option<&'a PendingReward> {
    pending_rewards.iter().find(|r| r.id == reward_id)
}

/// Removes a pending reward by ID and returns it if found
pub fn remove_pending_reward_by_id(
    pending_rewards: &mut Vec<PendingReward>,
    reward_id: &str,
) -> Option<PendingReward> {
    if let Some(index) = pending_rewards.iter().position(|r| r.id == reward_id) {
        Some(pending_rewards.remove(index))
    } else {
        None
    }
}

/// Checks if a user can claim any rewards
pub fn has_claimable_rewards(missions: &UserDailyMissions, now: SystemTime) -> bool {
    let progress = build_mission_progress(missions, now);

    progress.login_streak.can_claim
        || progress.game_progress.can_claim
        || progress.ai_video_progress.can_claim
        || progress.referral_progress.can_claim
}

/// Counts pending rewards by type
pub fn count_pending_rewards_by_type(
    pending_rewards: &[PendingReward],
    reward_type: &RewardType,
) -> usize {
    pending_rewards
        .iter()
        .filter(|r| &r.reward_type == reward_type)
        .count()
}

/// Gets the latest earned reward timestamp for a specific type
pub fn get_latest_reward_timestamp(
    pending_rewards: &[PendingReward],
    reward_type: &RewardType,
) -> Option<SystemTime> {
    pending_rewards
        .iter()
        .filter(|r| &r.reward_type == reward_type)
        .map(|r| r.earned_at)
        .max()
}

/// Validates if a mission progress state is consistent
pub fn validate_mission_progress(missions: &UserDailyMissions) -> Result<(), String> {
    // Validate login streak
    if missions.login_streak.current_streak > missions.login_streak.max_streak {
        return Err("Current streak cannot exceed max streak".to_string());
    }

    // Validate AI video count
    if missions.ai_video_count.videos_generated_total
        > missions.ai_video_count.total_videos_generated
    {
        return Err("Daily videos cannot exceed total videos".to_string());
    }

    // Validate referral count
    if missions.referral_count.referrals_made_total > missions.referral_count.total_referrals_made {
        return Err("Daily referrals cannot exceed total referrals".to_string());
    }

    // Validate referrals list consistency
    if missions.referral_count.referrals_made_total
        != missions.referral_count.referred_users.len() as u32
    {
        return Err("Referral count mismatch with referred users list".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::{
        GameStreak, GenerateAiVideoCount, LoginStreak, ReferralCount, ReferredUser,
    };
    use candid::Principal;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn create_test_time(days: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(days * 24 * 60 * 60)
    }

    fn create_test_missions() -> UserDailyMissions {
        UserDailyMissions {
            login_streak: LoginStreak {
                current_streak: 5,
                max_streak: 10,
                last_login_date: Some(create_test_time(1)),
                streak_start_date: Some(create_test_time(1)),
                claimed_rewards: vec![],
                can_claim_today: true,
            },
            game_streak: GameStreak {
                games_played_today: 3,
                target_games: 5,
                claimed_today: false,
                last_reset_date: Some(create_test_time(1)),
            },
            ai_video_count: GenerateAiVideoCount {
                videos_generated_total: 2,
                total_videos_generated: 2,
                target_videos: 3,
                completed: false,
            },
            referral_count: ReferralCount {
                referrals_made_total: 1,
                total_referrals_made: 1,
                target_referrals: 3,
                completed: false,
                referred_users: vec![ReferredUser {
                    principal_id: Principal::from_slice(&[1, 2, 3, 4]),
                    referred_at: create_test_time(1),
                    is_active: true,
                }],
            },
            last_updated: create_test_time(1),
            pending_rewards: vec![],
        }
    }

    #[test]
    fn test_build_mission_progress() {
        let missions = create_test_missions();
        let now = create_test_time(2);

        let progress = build_mission_progress(&missions, now);

        assert_eq!(progress.login_streak.current_day, 5);
        assert_eq!(progress.login_streak.target_day, LOGIN_STREAK_TARGET);
        assert_eq!(progress.game_progress.current_count, 3);
        assert_eq!(progress.game_progress.target_count, 5);
        assert_eq!(progress.ai_video_progress.current_count, 2);
        assert_eq!(progress.ai_video_progress.target_count, 3);
        assert_eq!(progress.referral_progress.current_count, 1);
        assert_eq!(progress.referral_progress.target_count, 3);
    }

    #[test]
    fn test_calculate_total_pending_rewards() {
        let pending_rewards = vec![
            PendingReward {
                id: "test1".to_string(),
                reward_type: RewardType::LoginStreak,
                amount: 100,
                earned_at: create_test_time(1),
                mission_day: 1,
            },
            PendingReward {
                id: "test2".to_string(),
                reward_type: RewardType::GameCompletion,
                amount: 200,
                earned_at: create_test_time(1),
                mission_day: 1,
            },
        ];

        let total = calculate_total_pending_rewards(&pending_rewards);
        assert_eq!(total, 300);
    }

    #[test]
    fn test_find_pending_reward_by_id() {
        let pending_rewards = vec![PendingReward {
            id: "test1".to_string(),
            reward_type: RewardType::LoginStreak,
            amount: 100,
            earned_at: create_test_time(1),
            mission_day: 1,
        }];

        let found = find_pending_reward_by_id(&pending_rewards, "test1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().amount, 100);

        let not_found = find_pending_reward_by_id(&pending_rewards, "test2");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_remove_pending_reward_by_id() {
        let mut pending_rewards = vec![
            PendingReward {
                id: "test1".to_string(),
                reward_type: RewardType::LoginStreak,
                amount: 100,
                earned_at: create_test_time(1),
                mission_day: 1,
            },
            PendingReward {
                id: "test2".to_string(),
                reward_type: RewardType::GameCompletion,
                amount: 200,
                earned_at: create_test_time(1),
                mission_day: 1,
            },
        ];

        let removed = remove_pending_reward_by_id(&mut pending_rewards, "test1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().amount, 100);
        assert_eq!(pending_rewards.len(), 1);
        assert_eq!(pending_rewards[0].id, "test2");

        let not_found = remove_pending_reward_by_id(&mut pending_rewards, "test3");
        assert!(not_found.is_none());
        assert_eq!(pending_rewards.len(), 1);
    }

    #[test]
    fn test_has_claimable_rewards() {
        let mut missions = create_test_missions();
        let now = create_test_time(2);

        // Initially should have claimable login streak reward
        missions.login_streak.can_claim_today = true;
        missions.login_streak.current_streak = LOGIN_STREAK_TARGET;
        assert!(has_claimable_rewards(&missions, now));

        // Remove claimability
        missions.login_streak.can_claim_today = false;
        missions.login_streak.current_streak = 1;
        assert!(!has_claimable_rewards(&missions, now));
    }

    #[test]
    fn test_count_pending_rewards_by_type() {
        let pending_rewards = vec![
            PendingReward {
                id: "test1".to_string(),
                reward_type: RewardType::LoginStreak,
                amount: 100,
                earned_at: create_test_time(1),
                mission_day: 1,
            },
            PendingReward {
                id: "test2".to_string(),
                reward_type: RewardType::LoginStreak,
                amount: 100,
                earned_at: create_test_time(2),
                mission_day: 2,
            },
            PendingReward {
                id: "test3".to_string(),
                reward_type: RewardType::GameCompletion,
                amount: 200,
                earned_at: create_test_time(1),
                mission_day: 1,
            },
        ];

        let login_count = count_pending_rewards_by_type(&pending_rewards, &RewardType::LoginStreak);
        assert_eq!(login_count, 2);

        let game_count =
            count_pending_rewards_by_type(&pending_rewards, &RewardType::GameCompletion);
        assert_eq!(game_count, 1);

        let ai_count =
            count_pending_rewards_by_type(&pending_rewards, &RewardType::AiVideoGeneration);
        assert_eq!(ai_count, 0);
    }

    #[test]
    fn test_validate_mission_progress() {
        let mut missions = create_test_missions();

        // Valid state
        assert!(validate_mission_progress(&missions).is_ok());

        // Invalid: current streak > max streak
        missions.login_streak.current_streak = 15;
        missions.login_streak.max_streak = 10;
        assert!(validate_mission_progress(&missions).is_err());

        // Fix and test another invalid state
        missions.login_streak.current_streak = 5;
        missions.login_streak.max_streak = 10;
        missions.ai_video_count.videos_generated_total = 5;
        missions.ai_video_count.total_videos_generated = 3;
        assert!(validate_mission_progress(&missions).is_err());
    }

    #[test]
    fn test_get_latest_reward_timestamp() {
        let pending_rewards = vec![
            PendingReward {
                id: "test1".to_string(),
                reward_type: RewardType::LoginStreak,
                amount: 100,
                earned_at: create_test_time(1),
                mission_day: 1,
            },
            PendingReward {
                id: "test2".to_string(),
                reward_type: RewardType::LoginStreak,
                amount: 100,
                earned_at: create_test_time(3),
                mission_day: 3,
            },
            PendingReward {
                id: "test3".to_string(),
                reward_type: RewardType::GameCompletion,
                amount: 200,
                earned_at: create_test_time(2),
                mission_day: 1,
            },
        ];

        let latest_login = get_latest_reward_timestamp(&pending_rewards, &RewardType::LoginStreak);
        assert_eq!(latest_login, Some(create_test_time(3)));

        let latest_game =
            get_latest_reward_timestamp(&pending_rewards, &RewardType::GameCompletion);
        assert_eq!(latest_game, Some(create_test_time(2)));

        let latest_ai =
            get_latest_reward_timestamp(&pending_rewards, &RewardType::AiVideoGeneration);
        assert_eq!(latest_ai, None);
    }
}
