use candid::{CandidType, Principal};
use ic_cdk::{caller, query, update};
use serde::{Deserialize, Serialize};

use crate::data_model::{
    ClaimedReward, MissionProgress, MissionType, MissionUpdateResult, RewardType,
    UserDailyMissions, AI_VIDEO_GENERATION_REWARD, DAILY_LOGIN_REWARD, GAME_COMPLETION_REWARD,
    LOGIN_STREAK_COMPLETION_BONUS, LOGIN_STREAK_TARGET, REFERRAL_REWARD,
};
use crate::util::mission_utils::{
    add_pending_reward, update_ai_video_count, update_game_streak, update_login_streak,
    update_referral_count,
};
use crate::util::progress_utils::{build_mission_progress, remove_pending_reward_by_id};
use shared_utils::common::utils::system_time::get_current_system_time_from_ic;

use crate::CANISTER_DATA;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UpdateMissionRequest {
    pub mission_type: MissionType,
    pub data: MissionUpdateData,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum MissionUpdateData {
    LoginEvent,
    GamePlayed { game_id: String },
    AiVideoGenerated { video_id: String },
    ReferralMade { referred_user: Principal },
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ClaimRewardRequest {
    pub reward_id: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ClaimRewardResponse {
    pub success: bool,
    pub message: String,
    pub reward_amount: Option<u64>,
    pub claimed_reward: Option<ClaimedReward>,
}

// Query endpoints
#[query]
fn get_current_missions() -> UserDailyMissions {
    let user = caller();
    CANISTER_DATA.with_borrow(|canister_data| canister_data.get_user_missions(&user))
}

#[query]
fn get_current_mission_progress() -> MissionProgress {
    let user = caller();
    let now = get_current_system_time_from_ic();
    CANISTER_DATA.with_borrow(|canister_data| {
        let missions = canister_data.get_user_missions(&user);
        build_mission_progress(&missions, now)
    })
}

#[query]
fn get_user_missions_for_principal(principal: Principal) -> UserDailyMissions {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.get_user_missions(&principal))
}

// Update endpoints
#[update]
fn update_mission(request: UpdateMissionRequest) -> MissionUpdateResult {
    let user = caller();
    let now = get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut missions = canister_data.get_user_missions(&user);
        missions.last_updated = now;
        let reward_earned = None;

        let result = match request.mission_type {
            MissionType::LoginStreak => {
                let prev_streak = missions.login_streak.current_streak;

                if let MissionUpdateData::LoginEvent = request.data {
                    let result = update_login_streak(&mut missions.login_streak, now);

                    // Add daily login reward if this is a new login (streak increased or started)
                    if result.0 && missions.login_streak.current_streak > prev_streak {
                        let current_streak = missions.login_streak.current_streak;
                        add_pending_reward(
                            &mut missions,
                            &user,
                            RewardType::LoginStreak,
                            DAILY_LOGIN_REWARD,
                            current_streak,
                            now,
                        );
                    }

                    // Add streak completion bonus when reaching 7-day target
                    let current_streak = missions.login_streak.current_streak;
                    if current_streak == LOGIN_STREAK_TARGET && prev_streak < LOGIN_STREAK_TARGET {
                        add_pending_reward(
                            &mut missions,
                            &user,
                            RewardType::LoginStreakBonus,
                            LOGIN_STREAK_COMPLETION_BONUS,
                            LOGIN_STREAK_TARGET,
                            now,
                        );
                    }
                    result
                } else {
                    return MissionUpdateResult {
                        success: false,
                        message: "Invalid data for login streak mission".to_string(),
                        new_progress: None,
                        reward_earned: None,
                    };
                }
            }
            MissionType::PlayGames => {
                let prev_games = missions.game_streak.games_played_today;
                let prev_claimed = missions.game_streak.claimed_today;

                if let MissionUpdateData::GamePlayed { .. } = request.data {
                    let result = update_game_streak(&mut missions.game_streak, now);

                    // Check if we should add a pending reward
                    if !prev_claimed
                        && missions.game_streak.games_played_today
                            >= missions.game_streak.target_games
                        && prev_games < missions.game_streak.target_games
                    {
                        add_pending_reward(
                            &mut missions,
                            &user,
                            RewardType::GameCompletion,
                            GAME_COMPLETION_REWARD,
                            1,
                            now,
                        );
                    }
                    result
                } else {
                    return MissionUpdateResult {
                        success: false,
                        message: "Invalid data for game mission".to_string(),
                        new_progress: None,
                        reward_earned: None,
                    };
                }
            }
            MissionType::GenerateAiVideos => {
                let prev_completed = missions.ai_video_count.completed;

                if let MissionUpdateData::AiVideoGenerated { .. } = request.data {
                    let result = update_ai_video_count(&mut missions.ai_video_count, now);

                    // Check if we should add a pending reward
                    if !prev_completed
                        && missions.ai_video_count.completed
                        && !missions.ai_video_count.reward_claimed
                    {
                        add_pending_reward(
                            &mut missions,
                            &user,
                            RewardType::AiVideoGeneration,
                            AI_VIDEO_GENERATION_REWARD,
                            1,
                            now,
                        );
                    }
                    result
                } else {
                    return MissionUpdateResult {
                        success: false,
                        message: "Invalid data for AI video mission".to_string(),
                        new_progress: None,
                        reward_earned: None,
                    };
                }
            }
            MissionType::Referrals => {
                let prev_completed = missions.referral_count.completed;

                if let MissionUpdateData::ReferralMade { referred_user } = request.data {
                    let result =
                        update_referral_count(&mut missions.referral_count, referred_user, now);

                    // Check if we should add a pending reward
                    if !prev_completed
                        && missions.referral_count.completed
                        && !missions.referral_count.reward_claimed
                    {
                        add_pending_reward(
                            &mut missions,
                            &user,
                            RewardType::Referral,
                            REFERRAL_REWARD,
                            1,
                            now,
                        );
                    }
                    result
                } else {
                    return MissionUpdateResult {
                        success: false,
                        message: "Invalid data for referral mission".to_string(),
                        new_progress: None,
                        reward_earned: None,
                    };
                }
            }
        };

        canister_data.update_user_missions(user, missions.clone());

        MissionUpdateResult {
            success: result.0,
            message: result.1,
            new_progress: Some(build_mission_progress(&missions, now)),
            reward_earned,
        }
    })
}

#[update]
fn claim_reward(request: ClaimRewardRequest) -> ClaimRewardResponse {
    let user = caller();
    let now = get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut missions = canister_data.get_user_missions(&user);

        // Find and remove the pending reward with the given ID
        if let Some(pending_reward) =
            remove_pending_reward_by_id(&mut missions.pending_rewards, &request.reward_id)
        {
            let claimed_reward = ClaimedReward {
                reward_type: pending_reward.reward_type.clone(),
                amount: pending_reward.amount,
                claimed_at: now,
                day: pending_reward.mission_day,
            };

            match pending_reward.reward_type {
                RewardType::LoginStreak => {
                    missions
                        .login_streak
                        .claimed_rewards
                        .push(claimed_reward.clone());
                }
                RewardType::LoginStreakBonus => {
                    missions
                        .login_streak
                        .claimed_rewards
                        .push(claimed_reward.clone());
                }
                RewardType::GameCompletion => {
                    missions.game_streak.claimed_today = true;
                    missions.game_streak.total_games_completed += 1;
                }
                RewardType::AiVideoGeneration => {
                    missions.ai_video_count.reward_claimed = true;
                }
                RewardType::Referral => {
                    missions.referral_count.reward_claimed = true;
                }
            }

            canister_data.update_user_missions(user, missions);

            ClaimRewardResponse {
                success: true,
                message: format!(
                    "{:?} reward claimed successfully",
                    pending_reward.reward_type
                ),
                reward_amount: Some(pending_reward.amount),
                claimed_reward: Some(claimed_reward),
            }
        } else {
            ClaimRewardResponse {
                success: false,
                message: "Reward not found or already claimed".to_string(),
                reward_amount: None,
                claimed_reward: None,
            }
        }
    })
}
