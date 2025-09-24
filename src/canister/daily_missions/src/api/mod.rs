use candid::{CandidType, Principal};
use ic_cdk::{caller, query, update};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::data_model::{
    AiVideoProgress, ClaimedReward, GameProgress, GameStreak, GenerateAiVideoCount, LoginStreak,
    LoginStreakProgress, MissionProgress, MissionType, MissionUpdateResult, ReferralCount,
    ReferralProgress, ReferredUser, RewardType, UserDailyMissions, AI_VIDEO_GENERATION_REWARD,
    GAME_COMPLETION_REWARD, LOGIN_STREAK_REWARD, LOGIN_STREAK_TARGET, REFERRAL_REWARD,
};
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
    pub mission_type: MissionType,
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
fn get_current_missions() -> Result<MissionProgress, String> {
    let user = caller();

    CANISTER_DATA.with_borrow(|canister_data| {
        let missions = canister_data.get_user_missions(&user);
        Ok(build_mission_progress(&missions))
    })
}

#[query]
fn get_user_missions_for_principal(user: Principal) -> Result<MissionProgress, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let missions = canister_data.get_user_missions(&user);
        Ok(build_mission_progress(&missions))
    })
}

// Update endpoints
#[update]
fn update_mission(request: UpdateMissionRequest) -> MissionUpdateResult {
    let user = caller();
    let now = get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut missions = canister_data.get_user_missions(&user);
        missions.last_updated = now;

        let result = match request.mission_type {
            MissionType::LoginStreak => {
                if let MissionUpdateData::LoginEvent = request.data {
                    update_login_streak(&mut missions.login_streak, now)
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
                if let MissionUpdateData::GamePlayed { .. } = request.data {
                    update_game_streak(&mut missions.game_streak, now)
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
                if let MissionUpdateData::AiVideoGenerated { .. } = request.data {
                    update_ai_video_count(&mut missions.ai_video_count, now)
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
                if let MissionUpdateData::ReferralMade { referred_user } = request.data {
                    update_referral_count(&mut missions.referral_count, referred_user, now)
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
            new_progress: Some(build_mission_progress(&missions)),
            reward_earned: None,
        }
    })
}

#[update]
fn claim_reward(request: ClaimRewardRequest) -> ClaimRewardResponse {
    let user = caller();
    let now = get_current_system_time_from_ic();

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let mut missions = canister_data.get_user_missions(&user);

        let result = match request.mission_type {
            MissionType::LoginStreak => {
                if missions.login_streak.can_claim_today
                    && missions.login_streak.current_streak >= LOGIN_STREAK_TARGET
                {
                    let reward = ClaimedReward {
                        reward_type: RewardType::LoginStreak,
                        amount: LOGIN_STREAK_REWARD,
                        claimed_at: now,
                        day: missions.login_streak.current_streak,
                    };
                    missions.login_streak.claimed_rewards.push(reward.clone());
                    missions.login_streak.can_claim_today = false;

                    canister_data.update_user_missions(user, missions);

                    ClaimRewardResponse {
                        success: true,
                        message: "Login streak reward claimed successfully".to_string(),
                        reward_amount: Some(LOGIN_STREAK_REWARD),
                        claimed_reward: Some(reward),
                    }
                } else {
                    ClaimRewardResponse {
                        success: false,
                        message: "Login streak reward not available for claim".to_string(),
                        reward_amount: None,
                        claimed_reward: None,
                    }
                }
            }
            MissionType::PlayGames => {
                if !missions.game_streak.claimed_today
                    && missions.game_streak.games_played_today >= missions.game_streak.target_games
                {
                    let reward = ClaimedReward {
                        reward_type: RewardType::GameCompletion,
                        amount: GAME_COMPLETION_REWARD,
                        claimed_at: now,
                        day: 1,
                    };
                    missions.game_streak.claimed_today = true;
                    missions.game_streak.total_games_completed += 1;

                    canister_data.update_user_missions(user, missions);

                    ClaimRewardResponse {
                        success: true,
                        message: "Game completion reward claimed successfully".to_string(),
                        reward_amount: Some(GAME_COMPLETION_REWARD),
                        claimed_reward: Some(reward),
                    }
                } else {
                    ClaimRewardResponse {
                        success: false,
                        message: "Game completion reward not available for claim".to_string(),
                        reward_amount: None,
                        claimed_reward: None,
                    }
                }
            }
            MissionType::GenerateAiVideos => {
                if !missions.ai_video_count.claimed_today
                    && missions.ai_video_count.videos_generated_today
                        >= missions.ai_video_count.target_videos
                {
                    let reward = ClaimedReward {
                        reward_type: RewardType::AiVideoGeneration,
                        amount: AI_VIDEO_GENERATION_REWARD,
                        claimed_at: now,
                        day: 1,
                    };
                    missions.ai_video_count.claimed_today = true;
                    missions.ai_video_count.total_videos_generated +=
                        missions.ai_video_count.videos_generated_today;

                    canister_data.update_user_missions(user, missions);

                    ClaimRewardResponse {
                        success: true,
                        message: "AI video generation reward claimed successfully".to_string(),
                        reward_amount: Some(AI_VIDEO_GENERATION_REWARD),
                        claimed_reward: Some(reward),
                    }
                } else {
                    ClaimRewardResponse {
                        success: false,
                        message: "AI video generation reward not available for claim".to_string(),
                        reward_amount: None,
                        claimed_reward: None,
                    }
                }
            }
            MissionType::Referrals => {
                if !missions.referral_count.claimed_today
                    && missions.referral_count.referrals_made_today
                        >= missions.referral_count.target_referrals
                {
                    let reward = ClaimedReward {
                        reward_type: RewardType::Referral,
                        amount: REFERRAL_REWARD,
                        claimed_at: now,
                        day: 1,
                    };
                    missions.referral_count.claimed_today = true;
                    missions.referral_count.total_referrals_made +=
                        missions.referral_count.referrals_made_today;

                    canister_data.update_user_missions(user, missions);

                    ClaimRewardResponse {
                        success: true,
                        message: "Referral reward claimed successfully".to_string(),
                        reward_amount: Some(REFERRAL_REWARD),
                        claimed_reward: Some(reward),
                    }
                } else {
                    ClaimRewardResponse {
                        success: false,
                        message: "Referral reward not available for claim".to_string(),
                        reward_amount: None,
                        claimed_reward: None,
                    }
                }
            }
        };

        result
    })
}

// Helper functions
fn build_mission_progress(missions: &UserDailyMissions) -> MissionProgress {
    let now = get_current_system_time_from_ic();

    MissionProgress {
        login_streak: LoginStreakProgress {
            current_day: missions.login_streak.current_streak,
            target_day: LOGIN_STREAK_TARGET,
            can_claim: missions.login_streak.can_claim_today
                && missions.login_streak.current_streak >= LOGIN_STREAK_TARGET,
            reward_amount: LOGIN_STREAK_REWARD,
            next_reset_in_hours: calculate_hours_until_next_day(now),
        },
        game_progress: GameProgress {
            current_count: missions.game_streak.games_played_today,
            target_count: missions.game_streak.target_games,
            can_claim: !missions.game_streak.claimed_today
                && missions.game_streak.games_played_today >= missions.game_streak.target_games,
            reward_amount: GAME_COMPLETION_REWARD,
            hours_remaining: calculate_hours_until_reset(
                &missions.game_streak.last_reset_date,
                now,
            ),
        },
        ai_video_progress: AiVideoProgress {
            current_count: missions.ai_video_count.videos_generated_today,
            target_count: missions.ai_video_count.target_videos,
            can_claim: !missions.ai_video_count.claimed_today
                && missions.ai_video_count.videos_generated_today
                    >= missions.ai_video_count.target_videos,
            reward_amount: AI_VIDEO_GENERATION_REWARD,
            hours_remaining: calculate_hours_until_reset(
                &missions.ai_video_count.last_reset_date,
                now,
            ),
        },
        referral_progress: ReferralProgress {
            current_count: missions.referral_count.referrals_made_today,
            target_count: missions.referral_count.target_referrals,
            can_claim: !missions.referral_count.claimed_today
                && missions.referral_count.referrals_made_today
                    >= missions.referral_count.target_referrals,
            reward_amount: REFERRAL_REWARD,
            hours_remaining: calculate_hours_until_reset(
                &missions.referral_count.last_reset_date,
                now,
            ),
        },
    }
}

fn update_login_streak(login_streak: &mut LoginStreak, now: SystemTime) -> (bool, String) {
    let today = get_day_from_timestamp(now);

    if let Some(last_login) = login_streak.last_login_date {
        let last_login_day = get_day_from_timestamp(last_login);

        if today == last_login_day {
            return (true, "Already logged in today".to_string());
        }

        if today == last_login_day + 1 {
            // Consecutive day
            login_streak.current_streak += 1;
            login_streak.can_claim_today = true;
        } else {
            // Streak broken
            login_streak.current_streak = 1;
            login_streak.streak_start_date = Some(now);
            login_streak.can_claim_today = true;
        }
    } else {
        // First login
        login_streak.current_streak = 1;
        login_streak.streak_start_date = Some(now);
        login_streak.can_claim_today = true;
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

fn update_game_streak(game_streak: &mut GameStreak, now: SystemTime) -> (bool, String) {
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

fn update_ai_video_count(
    ai_video_count: &mut GenerateAiVideoCount,
    now: SystemTime,
) -> (bool, String) {
    if should_reset_daily_counter(&ai_video_count.last_reset_date, now) {
        ai_video_count.videos_generated_today = 0;
        ai_video_count.claimed_today = false;
        ai_video_count.last_reset_date = Some(now);
    }

    ai_video_count.videos_generated_today += 1;

    (
        true,
        format!(
            "AI videos generated today: {}/{}",
            ai_video_count.videos_generated_today, ai_video_count.target_videos
        ),
    )
}

fn update_referral_count(
    referral_count: &mut ReferralCount,
    referred_user: Principal,
    now: SystemTime,
) -> (bool, String) {
    // Check if user was already referred
    if referral_count
        .referred_users
        .iter()
        .any(|u| u.principal_id == referred_user)
    {
        return (false, "User already referred".to_string());
    }

    if should_reset_daily_counter(&referral_count.last_reset_date, now) {
        referral_count.referrals_made_today = 0;
        referral_count.claimed_today = false;
        referral_count.last_reset_date = Some(now);
    }

    referral_count.referrals_made_today += 1;
    referral_count.referred_users.push(ReferredUser {
        principal_id: referred_user,
        referred_at: now,
        is_active: true,
    });

    (
        true,
        format!(
            "Referrals made today: {}/{}",
            referral_count.referrals_made_today, referral_count.target_referrals
        ),
    )
}

fn should_reset_daily_counter(last_reset: &Option<SystemTime>, now: SystemTime) -> bool {
    match last_reset {
        Some(last) => {
            let last_day = get_day_from_timestamp(*last);
            let current_day = get_day_from_timestamp(now);
            current_day > last_day
        }
        None => true,
    }
}

fn get_day_from_timestamp(time: SystemTime) -> u64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / (24 * 60 * 60)
}

fn calculate_hours_until_next_day(now: SystemTime) -> u32 {
    let seconds_since_epoch = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let seconds_in_day = 24 * 60 * 60;
    let seconds_until_next_day = seconds_in_day - (seconds_since_epoch % seconds_in_day);
    (seconds_until_next_day / 3600) as u32
}

fn calculate_hours_until_reset(last_reset: &Option<SystemTime>, now: SystemTime) -> u32 {
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
