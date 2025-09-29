use crate::data_model::{
    GameStreak, GenerateAiVideoCount, LoginStreak, PendingReward, ReferralCount, ReferredUser,
    RewardType, UserDailyMissions,
};
use candid::Principal;
use std::time::SystemTime;

/// Updates login streak for a user based on current login
pub fn update_login_streak(login_streak: &mut LoginStreak, now: SystemTime) -> (bool, String) {
    let today = get_day_from_timestamp(now);

    if let Some(last_login) = login_streak.last_login_date {
        let last_login_day = get_day_from_timestamp(last_login);

        if today == last_login_day {
            return (false, "Already logged in today".to_string());
        }

        if today == last_login_day + 1 {
            // Consecutive day
            login_streak.current_streak += 1;
        } else {
            // Streak broken
            login_streak.current_streak = 1;
            login_streak.streak_start_date = Some(now);
        }
    } else {
        // First login
        login_streak.current_streak = 1;
        login_streak.streak_start_date = Some(now);
    }

    login_streak.last_login_date = Some(now);
    login_streak.max_streak = login_streak.max_streak.max(login_streak.current_streak);

    (
        true,
        format!(
            "Login streak updated to {} days",
            login_streak.current_streak
        ),
    )
}

/// Updates game streak for a user
pub fn update_game_streak(game_streak: &mut GameStreak, now: SystemTime) -> (bool, String) {
    if should_reset_daily_counter(&game_streak.last_reset_date, now) {
        game_streak.games_played_today = 0;
        game_streak.claimed_today = false;
        game_streak.last_reset_date = Some(now);
    }

    game_streak.games_played_today += 1;

    (
        true,
        format!(
            "Games played today: {}/{}",
            game_streak.games_played_today, game_streak.target_games
        ),
    )
}

/// Updates AI video generation count for a user
pub fn update_ai_video_count(
    ai_video_count: &mut GenerateAiVideoCount,
    now: SystemTime,
) -> (bool, String) {
    // Check if mission is already completed
    if ai_video_count.completed {
        return (
            false,
            "AI video generation mission already completed. No more rewards available.".to_string(),
        );
    }

    ai_video_count.videos_generated_total += 1;
    ai_video_count.total_videos_generated += 1;

    // Check if mission is now completed
    if ai_video_count.videos_generated_total >= ai_video_count.target_videos {
        ai_video_count.completed = true;
    }

    (
        true,
        format!(
            "AI videos generated: {}/{}{}",
            ai_video_count.videos_generated_total,
            ai_video_count.target_videos,
            if ai_video_count.completed {
                " - Mission Complete!"
            } else {
                ""
            }
        ),
    )
}

/// Updates referral count for a user
pub fn update_referral_count(
    referral_count: &mut ReferralCount,
    referred_user: Principal,
    now: SystemTime,
) -> (bool, String) {
    // Check if mission is already completed
    if referral_count.completed {
        return (
            false,
            "Referral mission already completed. No more rewards available.".to_string(),
        );
    }

    // Check if user was already referred
    if referral_count
        .referred_users
        .iter()
        .any(|u| u.principal_id == referred_user)
    {
        return (false, "User already referred".to_string());
    }

    referral_count.referrals_made_total += 1;
    referral_count.total_referrals_made += 1;
    referral_count.referred_users.push(ReferredUser {
        principal_id: referred_user,
        referred_at: now,
        is_active: true,
    });

    // Check if mission is now completed
    if referral_count.referrals_made_total >= referral_count.target_referrals {
        referral_count.completed = true;
    }

    (
        true,
        format!(
            "Referrals made: {}/{}{}",
            referral_count.referrals_made_total,
            referral_count.target_referrals,
            if referral_count.completed {
                " - Mission Complete!"
            } else {
                ""
            }
        ),
    )
}

/// Generates a unique reward ID
pub fn generate_reward_id(
    user: &Principal,
    reward_type: &RewardType,
    timestamp: SystemTime,
) -> String {
    format!(
        "{:?}_{:?}_{:?}",
        user.to_text(),
        reward_type,
        timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}

/// Adds a pending reward to user's missions
pub fn add_pending_reward(
    missions: &mut UserDailyMissions,
    user: &Principal,
    reward_type: RewardType,
    amount: u64,
    mission_day: u32,
    timestamp: SystemTime,
) {
    let reward_id = generate_reward_id(user, &reward_type, timestamp);
    let pending_reward = PendingReward {
        id: reward_id,
        reward_type,
        amount,
        earned_at: timestamp,
        mission_day,
    };
    missions.pending_rewards.push(pending_reward);
}

/// Checks if daily counter should be reset
pub fn should_reset_daily_counter(last_reset: &Option<SystemTime>, now: SystemTime) -> bool {
    match last_reset {
        Some(last) => {
            let last_day = get_day_from_timestamp(*last);
            let current_day = get_day_from_timestamp(now);
            current_day > last_day
        }
        None => true,
    }
}

/// Converts timestamp to day number since Unix epoch
pub fn get_day_from_timestamp(time: SystemTime) -> u64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / (24 * 60 * 60)
}

/// Calculates hours until next day (UTC)
pub fn calculate_hours_until_next_day(now: SystemTime) -> u32 {
    let seconds_since_epoch = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let seconds_in_day = 24 * 60 * 60;
    let seconds_until_next_day = seconds_in_day - (seconds_since_epoch % seconds_in_day);
    (seconds_until_next_day / 3600) as u32
}

/// Calculates hours until reset based on last reset time
pub fn calculate_hours_until_reset(last_reset: &Option<SystemTime>, now: SystemTime) -> u32 {
    match last_reset {
        Some(last) => {
            let hours_since_reset = now.duration_since(*last).unwrap().as_secs() / 3600;
            if hours_since_reset >= 24 {
                0
            } else {
                (24 - hours_since_reset) as u32
            }
        }
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn create_test_time(days: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(days * 24 * 60 * 60)
    }

    #[test]
    fn test_update_login_streak_first_login() {
        let mut streak = LoginStreak::default();
        let now = create_test_time(1);

        let (success, message) = update_login_streak(&mut streak, now);

        assert!(success);
        assert_eq!(streak.current_streak, 1);
        assert_eq!(streak.max_streak, 1);
        assert!(message.contains("Login streak updated to 1 days"));
    }

    #[test]
    fn test_update_login_streak_consecutive_days() {
        let mut streak = LoginStreak {
            current_streak: 1,
            max_streak: 1,
            last_login_date: Some(create_test_time(1)),
            streak_start_date: Some(create_test_time(1)),
            claimed_rewards: vec![],
        };
        let now = create_test_time(2);

        let (success, message) = update_login_streak(&mut streak, now);

        assert!(success);
        assert_eq!(streak.current_streak, 2);
        assert_eq!(streak.max_streak, 2);
        assert!(message.contains("Login streak updated to 2 days"));
    }

    #[test]
    fn test_update_login_streak_same_day() {
        let day1 = create_test_time(1);
        let mut streak = LoginStreak {
            current_streak: 1,
            max_streak: 1,
            last_login_date: Some(day1),
            streak_start_date: Some(day1),
            claimed_rewards: vec![],
        };
        // Same day, few hours later
        let now = day1 + Duration::from_secs(3600);

        let (success, message) = update_login_streak(&mut streak, now);

        assert!(!success); // Should return false for same day login
        assert_eq!(streak.current_streak, 1); // Should not increment
        assert_eq!(message, "Already logged in today");
    }

    #[test]
    fn test_update_login_streak_broken() {
        let mut streak = LoginStreak {
            current_streak: 5,
            max_streak: 5,
            last_login_date: Some(create_test_time(1)),
            streak_start_date: Some(create_test_time(1)),
            claimed_rewards: vec![],
        };
        let now = create_test_time(5); // 4 days gap

        let (success, message) = update_login_streak(&mut streak, now);

        assert!(success);
        assert_eq!(streak.current_streak, 1); // Reset to 1
        assert_eq!(streak.max_streak, 5); // Max should remain
        assert!(message.contains("Login streak updated to 1 days"));
    }

    #[test]
    fn test_update_ai_video_count_completion() {
        let mut count = GenerateAiVideoCount {
            videos_generated_total: 2,
            total_videos_generated: 2,
            target_videos: 3,
            completed: false,
            reward_claimed: false,
        };
        let now = create_test_time(1);

        let (success, message) = update_ai_video_count(&mut count, now);

        assert!(success);
        assert_eq!(count.videos_generated_total, 3);
        assert_eq!(count.total_videos_generated, 3);
        assert!(count.completed);
        assert!(message.contains("Mission Complete!"));
    }

    #[test]
    fn test_update_ai_video_count_already_completed() {
        let mut count = GenerateAiVideoCount {
            videos_generated_total: 3,
            total_videos_generated: 3,
            target_videos: 3,
            completed: true,
            reward_claimed: false,
        };
        let now = create_test_time(1);

        let (success, message) = update_ai_video_count(&mut count, now);

        assert!(!success);
        assert!(message.contains("already completed"));
        assert_eq!(count.videos_generated_total, 3); // Should not increment
    }

    #[test]
    fn test_get_day_from_timestamp() {
        let epoch = SystemTime::UNIX_EPOCH;
        let day1 = epoch + Duration::from_secs(24 * 60 * 60);
        let day2 = epoch + Duration::from_secs(2 * 24 * 60 * 60);

        assert_eq!(get_day_from_timestamp(epoch), 0);
        assert_eq!(get_day_from_timestamp(day1), 1);
        assert_eq!(get_day_from_timestamp(day2), 2);
    }

    #[test]
    fn test_should_reset_daily_counter() {
        let day1 = create_test_time(1);
        let day2 = create_test_time(2);

        // No previous reset - should reset
        assert!(should_reset_daily_counter(&None, day1));

        // Same day - should not reset
        assert!(!should_reset_daily_counter(&Some(day1), day1));

        // Different day - should reset
        assert!(should_reset_daily_counter(&Some(day1), day2));
    }

    #[test]
    fn test_calculate_hours_until_next_day() {
        // Test at the beginning of day 1 (should be 24 hours until next day)
        let start_of_day = create_test_time(1);
        let hours = calculate_hours_until_next_day(start_of_day);
        assert_eq!(hours, 24);

        // Test halfway through day 1 (should be 12 hours until next day)
        let mid_day = start_of_day + Duration::from_secs(12 * 60 * 60);
        let hours = calculate_hours_until_next_day(mid_day);
        assert_eq!(hours, 12);
    }
}
