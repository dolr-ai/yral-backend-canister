use candid::{CandidType, Deserialize, Principal};
use pocket_ic::PocketIc;
use shared_utils::canister_specific::individual_user_template::types::session::SessionType;
use test_utils::canister_calls::{query, update};
use test_utils::setup::env::pocket_ic_env::ServiceCanisters;
use test_utils::setup::test_constants::get_global_super_admin_principal_id;

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

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct PropertyRateLimitConfig {
    pub property: String,
    pub window_duration_seconds: u64,
    pub max_requests_per_window_registered: u64,
    pub max_requests_per_window_unregistered: u64,
}

/// Helper function to register a user in the user_info_service
pub fn register_user_for_testing(
    pocket_ic: &PocketIc,
    service_canisters: &ServiceCanisters,
    user_principal: Principal,
) -> Result<(), String> {
    let global_admin = get_global_super_admin_principal_id();

    update::<_, Result<(), String>>(
        pocket_ic,
        service_canisters.user_info_service_canister_id,
        global_admin,
        "register_new_user",
        (user_principal,),
    )
    .expect("Failed to call register_new_user")
}

#[derive(CandidType, Deserialize, Debug)]
pub enum GetSessionTypeResult {
    Ok(SessionType),
    Err(String),
}
