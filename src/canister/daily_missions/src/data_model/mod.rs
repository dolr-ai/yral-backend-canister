use candid::{CandidType, Principal};
use ic_stable_structures::{storable::Bound, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use shared_utils::service::{GetVersion, SetVersion};
use std::borrow::Cow;
use std::time::SystemTime;

use crate::data_model::memory::Memory;

pub mod memory;

//This is the source of truth for a user's daily missions.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UserDailyMissions {
    pub login_streak: LoginStreak,
    pub game_streak: GameStreak,
    pub ai_video_count: GenerateAiVideoCount,
    pub referral_count: ReferralCount,
    pub last_updated: SystemTime,
    pub pending_rewards: Vec<PendingReward>,
}

impl Default for UserDailyMissions {
    fn default() -> Self {
        Self {
            login_streak: LoginStreak::default(),
            game_streak: GameStreak::default(),
            ai_video_count: GenerateAiVideoCount::default(),
            referral_count: ReferralCount::default(),
            last_updated: SystemTime::UNIX_EPOCH,
            pending_rewards: Vec::new(),
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LoginStreak {
    pub current_streak: u32,
    pub max_streak: u32,
    pub last_login_date: Option<SystemTime>,
    pub streak_start_date: Option<SystemTime>,
    pub claimed_rewards: Vec<ClaimedReward>,
}

impl Default for LoginStreak {
    fn default() -> Self {
        Self {
            current_streak: 0,
            max_streak: 0,
            last_login_date: None,
            streak_start_date: None,
            claimed_rewards: Vec::new(),
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct GameStreak {
    pub games_played_today: u32,
    pub target_games: u32,
    pub last_reset_date: Option<SystemTime>,
    pub claimed_today: bool,
    pub total_games_completed: u32,
}

impl Default for GameStreak {
    fn default() -> Self {
        Self {
            games_played_today: 0,
            target_games: 10,
            last_reset_date: None,
            claimed_today: false,
            total_games_completed: 0,
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct GenerateAiVideoCount {
    pub videos_generated_total: u32,
    pub target_videos: u32,
    pub completed: bool,
    pub reward_claimed: bool,
    pub total_videos_generated: u32,
}

impl Default for GenerateAiVideoCount {
    fn default() -> Self {
        Self {
            videos_generated_total: 0,
            target_videos: 3,
            completed: false,
            reward_claimed: false,
            total_videos_generated: 0,
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ReferralCount {
    pub referrals_made_total: u32,
    pub target_referrals: u32,
    pub completed: bool,
    pub reward_claimed: bool,
    pub total_referrals_made: u32,
    pub referred_users: Vec<ReferredUser>,
}

impl Default for ReferralCount {
    fn default() -> Self {
        Self {
            referrals_made_total: 0,
            target_referrals: 3,
            completed: false,
            reward_claimed: false,
            total_referrals_made: 0,
            referred_users: Vec::new(),
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ReferredUser {
    pub principal_id: Principal,
    pub referred_at: SystemTime,
    pub is_active: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ClaimedReward {
    pub reward_type: RewardType,
    pub amount: u64,
    pub claimed_at: SystemTime,
    pub day: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct PendingReward {
    pub id: String,
    pub reward_type: RewardType,
    pub amount: u64,
    pub earned_at: SystemTime,
    pub mission_day: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RewardType {
    LoginStreak,
    LoginStreakBonus,
    GameCompletion,
    AiVideoGeneration,
    Referral,
}

//This struct is a simplified representation of a user's daily missions progress for clients
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MissionProgress {
    pub login_streak: LoginStreakProgress,
    pub game_progress: GameProgress,
    pub ai_video_progress: AiVideoProgress,
    pub referral_progress: ReferralProgress,
    pub pending_rewards: Vec<PendingReward>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LoginStreakProgress {
    pub current_day: u32,
    pub target_day: u32,
    pub can_claim: bool,
    pub reward_amount: u64,
    pub next_reset_in_hours: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct GameProgress {
    pub current_count: u32,
    pub target_count: u32,
    pub can_claim: bool,
    pub reward_amount: u64,
    pub hours_remaining: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct AiVideoProgress {
    pub current_count: u32,
    pub target_count: u32,
    pub can_claim: bool,
    pub reward_amount: u64,
    pub completed: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ReferralProgress {
    pub current_count: u32,
    pub target_count: u32,
    pub can_claim: bool,
    pub reward_amount: u64,
    pub completed: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum MissionType {
    LoginStreak,
    PlayGames,
    GenerateAiVideos,
    Referrals,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct MissionUpdateResult {
    pub success: bool,
    pub message: String,
    pub new_progress: Option<UserDailyMissions>,
    pub reward_earned: Option<ClaimedReward>,
}

// Reward amounts in YRAL tokens
pub const DAILY_LOGIN_REWARD: u64 = 5;
pub const LOGIN_STREAK_COMPLETION_BONUS: u64 = 30;
pub const GAME_COMPLETION_REWARD: u64 = 10;
pub const AI_VIDEO_GENERATION_REWARD: u64 = 30;
pub const REFERRAL_REWARD: u64 = 15;

// Target values
pub const LOGIN_STREAK_TARGET: u32 = 7;
pub const GAME_TARGET: u32 = 10;
pub const AI_VIDEO_TARGET: u32 = 3;
pub const REFERRAL_TARGET: u32 = 3;

// Storable implementation for UserDailyMissions
impl Storable for UserDailyMissions {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = candid::encode_one(self).expect("Failed to encode UserDailyMissions");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).expect("Failed to decode UserDailyMissions")
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub struct CanisterData {
    pub missions: StableBTreeMap<Principal, UserDailyMissions, Memory>,
    pub version: String,
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            missions: StableBTreeMap::init(memory::get_stable_btree_memory()),
            version: "1.0.0".to_string(),
        }
    }
}

impl Serialize for CanisterData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CanisterData", 1)?;
        state.serialize_field("version", &self.version)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for CanisterData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TempCanisterData {
            version: String,
        }

        let temp = TempCanisterData::deserialize(deserializer)?;
        Ok(CanisterData {
            missions: StableBTreeMap::init(memory::get_stable_btree_memory()),
            version: temp.version,
        })
    }
}

impl CanisterData {
    pub fn new_with_version(version: String) -> Self {
        Self {
            missions: StableBTreeMap::init(memory::get_stable_btree_memory()),
            version,
        }
    }
}

impl SetVersion for CanisterData {
    fn set_version(&mut self, version: &str) {
        self.version = version.to_string();
    }
}

impl GetVersion for CanisterData {
    fn get_version(&self) -> Cow<str> {
        self.version.as_str().into()
    }
}

impl CanisterData {
    pub fn get_user_missions(&self, user: &Principal) -> UserDailyMissions {
        self.missions.get(user).unwrap_or_default()
    }

    pub fn update_user_missions(&mut self, user: Principal, missions: UserDailyMissions) {
        self.missions.insert(user, missions);
    }

    pub fn user_exists(&self, user: &Principal) -> bool {
        self.missions.contains_key(user)
    }
}
