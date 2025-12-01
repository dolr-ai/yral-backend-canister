use std::default;

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq)]
pub struct FollowerItem {
    pub principal_id: Principal,
    pub caller_follows: bool,
    pub profile_picture_url: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct ProfileUpdateDetails {
    pub bio: Option<String>,
    pub website_url: Option<String>,
    pub profile_picture_url: Option<String>,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct FollowersResponse {
    pub followers: Vec<FollowerItem>,
    pub total_count: u64,
    pub next_cursor: Option<Principal>,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct FollowingResponse {
    pub following: Vec<FollowerItem>,
    pub total_count: u64,
    pub next_cursor: Option<Principal>,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct YralProSubscription {
    pub free_video_credits_left: u32,
}

impl Default for YralProSubscription {
    fn default() -> Self {
        YralProSubscription {
            free_video_credits_left: 30,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub enum SubscriptionPlan {
    #[default]
    Free,
    Pro(YralProSubscription),
}
