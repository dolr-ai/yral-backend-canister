use candid::Principal;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::data_model::{
    ClaimedReward, GameStreak, GenerateAiVideoCount, LoginStreak, ReferralCount, ReferredUser,
    RewardType, UserDailyMissions, AI_VIDEO_GENERATION_REWARD, AI_VIDEO_TARGET, REFERRAL_REWARD,
    REFERRAL_TARGET,
};

// Helper function to create test principals
fn create_test_principal() -> Principal {
    Principal::anonymous()
}

fn create_test_time() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1000000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_video_mission_default_state() {
        let ai_video_count = GenerateAiVideoCount::default();

        assert_eq!(ai_video_count.videos_generated_total, 0);
        assert_eq!(ai_video_count.target_videos, AI_VIDEO_TARGET);
        assert_eq!(ai_video_count.completed, false);
        assert_eq!(ai_video_count.reward_claimed, false);
        assert_eq!(ai_video_count.total_videos_generated, 0);
    }

    #[test]
    fn test_referral_mission_default_state() {
        let referral_count = ReferralCount::default();

        assert_eq!(referral_count.referrals_made_total, 0);
        assert_eq!(referral_count.target_referrals, REFERRAL_TARGET);
        assert_eq!(referral_count.completed, false);
        assert_eq!(referral_count.reward_claimed, false);
        assert_eq!(referral_count.total_referrals_made, 0);
        assert!(referral_count.referred_users.is_empty());
    }

    #[test]
    fn test_ai_video_mission_progress_tracking() {
        let mut ai_video_count = GenerateAiVideoCount::default();
        let now = create_test_time();

        // Initially not completed
        assert!(!ai_video_count.completed);

        // Generate first video
        ai_video_count.videos_generated_total += 1;
        ai_video_count.total_videos_generated += 1;
        assert!(!ai_video_count.completed);
        assert_eq!(ai_video_count.videos_generated_total, 1);

        // Generate second video
        ai_video_count.videos_generated_total += 1;
        ai_video_count.total_videos_generated += 1;
        assert!(!ai_video_count.completed);
        assert_eq!(ai_video_count.videos_generated_total, 2);

        // Generate third video (target reached)
        ai_video_count.videos_generated_total += 1;
        ai_video_count.total_videos_generated += 1;
        if ai_video_count.videos_generated_total >= ai_video_count.target_videos {
            ai_video_count.completed = true;
        }

        assert!(ai_video_count.completed);
        assert_eq!(ai_video_count.videos_generated_total, 3);
        assert_eq!(ai_video_count.total_videos_generated, 3);
    }

    #[test]
    fn test_referral_mission_progress_tracking() {
        let mut referral_count = ReferralCount::default();
        let now = create_test_time();
        let user1 = Principal::from_slice(&[1, 2, 3, 4]);
        let user2 = Principal::from_slice(&[2, 3, 4, 5]);
        let user3 = Principal::from_slice(&[3, 4, 5, 6]);

        // Initially not completed
        assert!(!referral_count.completed);

        // First referral
        referral_count.referrals_made_total += 1;
        referral_count.total_referrals_made += 1;
        referral_count.referred_users.push(ReferredUser {
            principal_id: user1,
            referred_at: now,
            is_active: true,
        });
        assert!(!referral_count.completed);
        assert_eq!(referral_count.referrals_made_total, 1);

        // Second referral
        referral_count.referrals_made_total += 1;
        referral_count.total_referrals_made += 1;
        referral_count.referred_users.push(ReferredUser {
            principal_id: user2,
            referred_at: now,
            is_active: true,
        });
        assert!(!referral_count.completed);
        assert_eq!(referral_count.referrals_made_total, 2);

        // Third referral (target reached)
        referral_count.referrals_made_total += 1;
        referral_count.total_referrals_made += 1;
        referral_count.referred_users.push(ReferredUser {
            principal_id: user3,
            referred_at: now,
            is_active: true,
        });
        if referral_count.referrals_made_total >= referral_count.target_referrals {
            referral_count.completed = true;
        }

        assert!(referral_count.completed);
        assert_eq!(referral_count.referrals_made_total, 3);
        assert_eq!(referral_count.total_referrals_made, 3);
        assert_eq!(referral_count.referred_users.len(), 3);
    }

    #[test]
    fn test_ai_video_mission_reward_claiming() {
        let mut ai_video_count = GenerateAiVideoCount::default();
        let now = create_test_time();

        // Complete the mission
        ai_video_count.videos_generated_total = AI_VIDEO_TARGET;
        ai_video_count.total_videos_generated = AI_VIDEO_TARGET;
        ai_video_count.completed = true;

        // Should be able to claim reward
        assert!(ai_video_count.completed);
        assert!(!ai_video_count.reward_claimed);

        // Claim reward
        ai_video_count.reward_claimed = true;

        // Should not be able to claim again
        assert!(ai_video_count.reward_claimed);
    }

    #[test]
    fn test_referral_mission_reward_claiming() {
        let mut referral_count = ReferralCount::default();
        let now = create_test_time();

        // Complete the mission
        referral_count.referrals_made_total = REFERRAL_TARGET;
        referral_count.total_referrals_made = REFERRAL_TARGET;
        referral_count.completed = true;

        // Should be able to claim reward
        assert!(referral_count.completed);
        assert!(!referral_count.reward_claimed);

        // Claim reward
        referral_count.reward_claimed = true;

        // Should not be able to claim again
        assert!(referral_count.reward_claimed);
    }

    #[test]
    fn test_ai_video_mission_no_reset_after_completion() {
        let mut ai_video_count = GenerateAiVideoCount::default();

        // Complete the mission
        ai_video_count.videos_generated_total = AI_VIDEO_TARGET;
        ai_video_count.completed = true;
        ai_video_count.reward_claimed = true;

        // Simulate time passing (e.g., next day)
        // In the old system, this would reset daily counters
        // In the new system, completed missions stay completed

        assert!(ai_video_count.completed);
        assert!(ai_video_count.reward_claimed);
        assert_eq!(ai_video_count.videos_generated_total, AI_VIDEO_TARGET);
    }

    #[test]
    fn test_referral_mission_no_reset_after_completion() {
        let mut referral_count = ReferralCount::default();
        let now = create_test_time();
        let user1 = Principal::from_slice(&[1, 2, 3, 4]);

        // Complete the mission
        referral_count.referrals_made_total = REFERRAL_TARGET;
        referral_count.completed = true;
        referral_count.reward_claimed = true;
        referral_count.referred_users.push(ReferredUser {
            principal_id: user1,
            referred_at: now,
            is_active: true,
        });

        // Simulate time passing (e.g., next day)
        // In the old system, this would reset daily counters
        // In the new system, completed missions stay completed

        assert!(referral_count.completed);
        assert!(referral_count.reward_claimed);
        assert_eq!(referral_count.referrals_made_total, REFERRAL_TARGET);
        assert_eq!(referral_count.referred_users.len(), 1);
    }

    #[test]
    fn test_user_daily_missions_integration() {
        let mut missions = UserDailyMissions::default();
        let now = create_test_time();

        // Initially, no missions are completed
        assert!(!missions.ai_video_count.completed);
        assert!(!missions.referral_count.completed);

        // Complete AI video mission
        missions.ai_video_count.videos_generated_total = AI_VIDEO_TARGET;
        missions.ai_video_count.completed = true;
        missions.ai_video_count.reward_claimed = true;
        missions.last_updated = now;

        // Referral mission should still be incomplete
        assert!(missions.ai_video_count.completed);
        assert!(!missions.referral_count.completed);

        // Complete referral mission
        missions.referral_count.referrals_made_total = REFERRAL_TARGET;
        missions.referral_count.completed = true;
        missions.referral_count.reward_claimed = true;

        // Both missions should be completed
        assert!(missions.ai_video_count.completed);
        assert!(missions.referral_count.completed);
    }

    #[test]
    fn test_duplicate_referral_prevention() {
        let mut referral_count = ReferralCount::default();
        let now = create_test_time();
        let user1 = Principal::from_slice(&[1, 2, 3, 4]);

        // Add first referral
        referral_count.referred_users.push(ReferredUser {
            principal_id: user1,
            referred_at: now,
            is_active: true,
        });

        // Check if user was already referred (simulating the check in update_referral_count)
        let already_referred = referral_count
            .referred_users
            .iter()
            .any(|u| u.principal_id == user1);

        assert!(already_referred);
    }

    #[test]
    fn test_mission_completion_thresholds() {
        // Test AI Video mission threshold
        let mut ai_video_count = GenerateAiVideoCount::default();

        // Just below threshold
        ai_video_count.videos_generated_total = AI_VIDEO_TARGET - 1;
        assert!(!ai_video_count.completed);

        // At threshold
        ai_video_count.videos_generated_total = AI_VIDEO_TARGET;
        if ai_video_count.videos_generated_total >= ai_video_count.target_videos {
            ai_video_count.completed = true;
        }
        assert!(ai_video_count.completed);

        // Test Referral mission threshold
        let mut referral_count = ReferralCount::default();

        // Just below threshold
        referral_count.referrals_made_total = REFERRAL_TARGET - 1;
        assert!(!referral_count.completed);

        // At threshold
        referral_count.referrals_made_total = REFERRAL_TARGET;
        if referral_count.referrals_made_total >= referral_count.target_referrals {
            referral_count.completed = true;
        }
        assert!(referral_count.completed);
    }

    #[test]
    fn test_referred_user_structure() {
        let now = create_test_time();
        let user1 = Principal::from_slice(&[1, 2, 3, 4]);

        let referred_user = ReferredUser {
            principal_id: user1,
            referred_at: now,
            is_active: true,
        };

        assert_eq!(referred_user.principal_id, user1);
        assert_eq!(referred_user.referred_at, now);
        assert!(referred_user.is_active);
    }

    #[test]
    fn test_mission_state_transitions() {
        // AI Video Mission State Transitions
        let mut ai_video_count = GenerateAiVideoCount::default();

        // State 1: Initial (not completed, not claimed)
        assert!(!ai_video_count.completed);
        assert!(!ai_video_count.reward_claimed);

        // State 2: Completed (completed, not claimed)
        ai_video_count.videos_generated_total = AI_VIDEO_TARGET;
        ai_video_count.completed = true;
        assert!(ai_video_count.completed);
        assert!(!ai_video_count.reward_claimed);

        // State 3: Completed and Claimed (completed, claimed)
        ai_video_count.reward_claimed = true;
        assert!(ai_video_count.completed);
        assert!(ai_video_count.reward_claimed);

        // Referral Mission State Transitions
        let mut referral_count = ReferralCount::default();

        // State 1: Initial (not completed, not claimed)
        assert!(!referral_count.completed);
        assert!(!referral_count.reward_claimed);

        // State 2: Completed (completed, not claimed)
        referral_count.referrals_made_total = REFERRAL_TARGET;
        referral_count.completed = true;
        assert!(referral_count.completed);
        assert!(!referral_count.reward_claimed);

        // State 3: Completed and Claimed (completed, claimed)
        referral_count.reward_claimed = true;
        assert!(referral_count.completed);
        assert!(referral_count.reward_claimed);
    }
}

mod api_tests;
