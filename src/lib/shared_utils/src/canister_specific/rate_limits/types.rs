use crate::service::GetVersion;
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;
use std::borrow::Cow;

#[derive(CandidType, Deserialize)]
pub struct RateLimitsInitArgs {
    pub version: String,
    pub user_info_canister: Principal,
}

impl GetVersion for RateLimitsInitArgs {
    fn get_version(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.version)
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct GlobalRateLimitConfig {
    pub max_requests_per_window_registered: u64,
    pub max_requests_per_window_unregistered: u64,
    pub window_duration_seconds: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct RateLimitConfig {
    pub max_requests_per_window: u64,
    pub window_duration_seconds: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct PropertyRateLimitConfig {
    pub property: String,
    pub max_requests_per_window_registered: u64,
    pub max_requests_per_window_unregistered: u64,
    pub window_duration_seconds: u64,
}

impl Storable for PropertyRateLimitConfig {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = serde_json::to_vec(&self).expect("Failed to serialize PropertyRateLimitConfig");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let inner: PropertyRateLimitConfig =
            serde_json::from_slice(&bytes).expect("Failed to deserialize PropertyRateLimitConfig");
        inner
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone)]
pub enum RateLimitResult {
    Ok(String),
    Err(String),
}

#[derive(CandidType, Deserialize, Clone)]
pub struct RateLimitStatus {
    pub principal: Principal,
    pub request_count: u64,
    pub window_start: u64,
    pub is_limited: bool,
}

impl Default for GlobalRateLimitConfig {
    fn default() -> Self {
        GlobalRateLimitConfig {
            max_requests_per_window_registered:
                super::consts::DEFAULT_MAX_REQUESTS_PER_WINDOW_REGISTERED,
            max_requests_per_window_unregistered:
                super::consts::DEFAULT_MAX_REQUESTS_PER_WINDOW_UNREGISTERED,
            window_duration_seconds: super::consts::DEFAULT_WINDOW_DURATION_SECONDS,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitKey {
    pub principal: Principal,
    pub property: String,
}

impl RateLimitKey {
    pub fn new(principal: Principal, property: String) -> Self {
        Self {
            principal,
            property,
        }
    }

    pub fn default_property(principal: Principal) -> Self {
        Self {
            principal,
            property: "default".to_string(),
        }
    }
}

impl Storable for RateLimitKey {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = serde_json::to_vec(self).expect("Failed to serialize RateLimitKey");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize RateLimitKey")
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RateLimitEntry {
    pub request_count: u64,
    pub window_start: u64,
    pub config: Option<RateLimitConfig>,
}

impl Storable for RateLimitEntry {
    fn to_bytes(&self) -> Cow<[u8]> {
        let bytes = serde_json::to_vec(self).expect("Failed to serialize RateLimitEntry");
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_json::from_slice(&bytes).expect("Failed to deserialize RateLimitEntry")
    }

    const BOUND: Bound = Bound::Unbounded;
}
