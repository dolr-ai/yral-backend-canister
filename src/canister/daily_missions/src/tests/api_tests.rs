use candid::Principal;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::api::{update_ai_video_count, update_referral_count};
use crate::data_model::{GenerateAiVideoCount, ReferralCount};

fn create_test_time() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1000000)
}

#[cfg(test)]
mod api_integration_tests {
    use super::*;

    #[test]
    fn test_update_ai_video_count_function() {
        let mut ai_video_count = GenerateAiVideoCount::default();
        let now = create_test_time();

        // First update - should succeed
        let (success, message) = update_ai_video_count(&mut ai_video_count, now);
        assert!(success);
        assert_eq!(ai_video_count.videos_generated_total, 1);
        assert!(!ai_video_count.completed);
        assert!(message.contains("AI videos generated: 1/3"));

        // Second update - should succeed
        let (success, _) = update_ai_video_count(&mut ai_video_count, now);
        assert!(success);
        assert_eq!(ai_video_count.videos_generated_total, 2);
        assert!(!ai_video_count.completed);

        // Third update - should succeed and complete mission
        let (success, message) = update_ai_video_count(&mut ai_video_count, now);
        assert!(success);
        assert_eq!(ai_video_count.videos_generated_total, 3);
        assert!(ai_video_count.completed);
        assert!(message.contains("Mission Complete!"));

        // Try to update after completion - should fail
        let (success, message) = update_ai_video_count(&mut ai_video_count, now);
        assert!(!success);
        assert!(message.contains("already completed"));
        assert!(message.contains("No more rewards available"));
    }

    #[test]
    fn test_update_referral_count_function() {
        let mut referral_count = ReferralCount::default();
        let now = create_test_time();
        let user1 = Principal::from_slice(&[1, 2, 3, 4]);
        let user2 = Principal::from_slice(&[2, 3, 4, 5]);
        let user3 = Principal::from_slice(&[3, 4, 5, 6]);

        // First referral - should succeed
        let (success, message) = update_referral_count(&mut referral_count, user1, now);
        assert!(success);
        assert_eq!(referral_count.referrals_made_total, 1);
        assert!(!referral_count.completed);
        assert!(message.contains("Referrals made: 1/3"));
        assert_eq!(referral_count.referred_users.len(), 1);

        // Duplicate referral - should fail
        let (success, message) = update_referral_count(&mut referral_count, user1, now);
        assert!(!success);
        assert!(message.contains("already referred"));
        // Counts should not change
        assert_eq!(referral_count.referrals_made_total, 1);
        assert_eq!(referral_count.referred_users.len(), 1);

        // Second referral with different user - should succeed
        let (success, _) = update_referral_count(&mut referral_count, user2, now);
        assert!(success);
        assert_eq!(referral_count.referrals_made_total, 2);
        assert!(!referral_count.completed);
        assert_eq!(referral_count.referred_users.len(), 2);

        // Third referral - should succeed and complete mission
        let (success, message) = update_referral_count(&mut referral_count, user3, now);
        assert!(success);
        assert_eq!(referral_count.referrals_made_total, 3);
        assert!(referral_count.completed);
        assert!(message.contains("Mission Complete!"));
        assert_eq!(referral_count.referred_users.len(), 3);

        // Try to refer after completion - should fail
        let user4 = Principal::from_slice(&[4, 5, 6, 7]);
        let (success, message) = update_referral_count(&mut referral_count, user4, now);
        assert!(!success);
        assert!(message.contains("already completed"));
        assert!(message.contains("No more rewards available"));
        // Counts should not change
        assert_eq!(referral_count.referrals_made_total, 3);
        assert_eq!(referral_count.referred_users.len(), 3);
    }

    #[test]
    fn test_ai_video_count_tracks_total_correctly() {
        let mut ai_video_count = GenerateAiVideoCount::default();
        let now = create_test_time();

        // Initially both counters should be 0
        assert_eq!(ai_video_count.videos_generated_total, 0);
        assert_eq!(ai_video_count.total_videos_generated, 0);

        // After first update, both should increment
        update_ai_video_count(&mut ai_video_count, now);
        assert_eq!(ai_video_count.videos_generated_total, 1);
        assert_eq!(ai_video_count.total_videos_generated, 1);

        // After second update, both should increment
        update_ai_video_count(&mut ai_video_count, now);
        assert_eq!(ai_video_count.videos_generated_total, 2);
        assert_eq!(ai_video_count.total_videos_generated, 2);

        // After third update (completion), both should increment
        update_ai_video_count(&mut ai_video_count, now);
        assert_eq!(ai_video_count.videos_generated_total, 3);
        assert_eq!(ai_video_count.total_videos_generated, 3);
        assert!(ai_video_count.completed);
    }

    #[test]
    fn test_referral_count_tracks_total_correctly() {
        let mut referral_count = ReferralCount::default();
        let now = create_test_time();
        let user1 = Principal::from_slice(&[1, 2, 3, 4]);
        let user2 = Principal::from_slice(&[2, 3, 4, 5]);
        let user3 = Principal::from_slice(&[3, 4, 5, 6]);

        // Initially both counters should be 0
        assert_eq!(referral_count.referrals_made_total, 0);
        assert_eq!(referral_count.total_referrals_made, 0);

        // After first referral, both should increment
        update_referral_count(&mut referral_count, user1, now);
        assert_eq!(referral_count.referrals_made_total, 1);
        assert_eq!(referral_count.total_referrals_made, 1);

        // After second referral, both should increment
        update_referral_count(&mut referral_count, user2, now);
        assert_eq!(referral_count.referrals_made_total, 2);
        assert_eq!(referral_count.total_referrals_made, 2);

        // After third referral (completion), both should increment
        update_referral_count(&mut referral_count, user3, now);
        assert_eq!(referral_count.referrals_made_total, 3);
        assert_eq!(referral_count.total_referrals_made, 3);
        assert!(referral_count.completed);
    }

    #[test]
    fn test_referral_users_list_management() {
        let mut referral_count = ReferralCount::default();
        let now = create_test_time();
        let user1 = Principal::from_slice(&[1, 2, 3, 4]);
        let user2 = Principal::from_slice(&[2, 3, 4, 5]);

        // Initially no referred users
        assert!(referral_count.referred_users.is_empty());

        // After first referral, list should contain user1
        update_referral_count(&mut referral_count, user1, now);
        assert_eq!(referral_count.referred_users.len(), 1);
        assert_eq!(referral_count.referred_users[0].principal_id, user1);
        assert_eq!(referral_count.referred_users[0].referred_at, now);
        assert!(referral_count.referred_users[0].is_active);

        // After second referral, list should contain both users
        update_referral_count(&mut referral_count, user2, now);
        assert_eq!(referral_count.referred_users.len(), 2);
        assert_eq!(referral_count.referred_users[1].principal_id, user2);
        assert_eq!(referral_count.referred_users[1].referred_at, now);
        assert!(referral_count.referred_users[1].is_active);

        // Duplicate referral should not add to the list
        let initial_len = referral_count.referred_users.len();
        let (success, _) = update_referral_count(&mut referral_count, user1, now);
        assert!(!success);
        assert_eq!(referral_count.referred_users.len(), initial_len);
    }

    #[test]
    fn test_mission_completion_states() {
        let mut ai_video_count = GenerateAiVideoCount::default();
        let mut referral_count = ReferralCount::default();
        let now = create_test_time();

        // Both missions start incomplete
        assert!(!ai_video_count.completed);
        assert!(!referral_count.completed);

        // AI video mission - complete all 3 videos
        for _ in 0..3 {
            update_ai_video_count(&mut ai_video_count, now);
        }
        assert!(ai_video_count.completed);

        // Referral mission - complete all 3 referrals
        let users = [
            Principal::from_slice(&[1, 2, 3, 4]),
            Principal::from_slice(&[2, 3, 4, 5]),
            Principal::from_slice(&[3, 4, 5, 6]),
        ];
        for user in users.iter() {
            update_referral_count(&mut referral_count, *user, now);
        }
        assert!(referral_count.completed);

        // Both missions should remain completed and reject further updates
        let (success, _) = update_ai_video_count(&mut ai_video_count, now);
        assert!(!success);

        let user4 = Principal::from_slice(&[4, 5, 6, 7]);
        let (success, _) = update_referral_count(&mut referral_count, user4, now);
        assert!(!success);
    }

    #[test]
    fn test_pending_rewards_generation() {
        use crate::api::{add_pending_reward, generate_reward_id};
        use crate::data_model::{
            RewardType, UserDailyMissions, AI_VIDEO_GENERATION_REWARD, LOGIN_STREAK_REWARD,
        };

        let mut missions = UserDailyMissions::default();
        let user = Principal::from_slice(&[1, 2, 3, 4]);
        let now = create_test_time();

        // Initially no pending rewards
        assert!(missions.pending_rewards.is_empty());

        // Add login streak reward
        add_pending_reward(
            &mut missions,
            &user,
            RewardType::LoginStreak,
            LOGIN_STREAK_REWARD,
            7,
            now,
        );

        // Should have one pending reward
        assert_eq!(missions.pending_rewards.len(), 1);
        assert_eq!(
            missions.pending_rewards[0].reward_type,
            RewardType::LoginStreak
        );
        assert_eq!(missions.pending_rewards[0].amount, LOGIN_STREAK_REWARD);
        assert_eq!(missions.pending_rewards[0].mission_day, 7);
        assert_eq!(missions.pending_rewards[0].earned_at, now);

        // Add AI video reward
        add_pending_reward(
            &mut missions,
            &user,
            RewardType::AiVideoGeneration,
            AI_VIDEO_GENERATION_REWARD,
            1,
            now,
        );

        // Should have two pending rewards
        assert_eq!(missions.pending_rewards.len(), 2);
        assert_eq!(
            missions.pending_rewards[1].reward_type,
            RewardType::AiVideoGeneration
        );
        assert_eq!(
            missions.pending_rewards[1].amount,
            AI_VIDEO_GENERATION_REWARD
        );
    }

    #[test]
    fn test_reward_id_generation() {
        use crate::api::generate_reward_id;
        use crate::data_model::RewardType;

        let user = Principal::from_slice(&[1, 2, 3, 4]);
        let now = create_test_time();

        let id1 = generate_reward_id(&user, &RewardType::LoginStreak, now);
        let id2 = generate_reward_id(&user, &RewardType::GameCompletion, now);

        // IDs should be different for different reward types
        assert_ne!(id1, id2);

        // IDs should contain expected components
        assert!(id1.contains("LoginStreak"));
        assert!(id2.contains("GameCompletion"));
    }
}
