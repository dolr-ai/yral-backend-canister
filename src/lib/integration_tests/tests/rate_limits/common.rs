use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum RateLimitResult {
    Ok(String),
    Err(String),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RateLimitStatus {
    pub principal: Principal,
    pub request_count: u64,
    pub window_start: u64,
    pub is_limited: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RateLimitConfig {
    pub window_duration_seconds: u64,
    pub max_requests_per_window: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct GlobalRateLimitConfig {
    pub window_duration_seconds: u64,
    pub max_requests_per_window_registered: u64,
    pub max_requests_per_window_unregistered: u64,
}