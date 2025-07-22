use candid::{CandidType, Deserialize, Principal};
use pocket_ic::PocketIc;
use test_utils::canister_calls::{update, query};
use test_utils::setup::env::pocket_ic_env::ServiceCanisters;
use test_utils::setup::test_constants::get_global_super_admin_principal_id;
use shared_utils::canister_specific::individual_user_template::types::session::SessionType;

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

/// Helper function to get the session type for a principal
pub fn get_session_type_for_principal(
    pocket_ic: &PocketIc,
    service_canisters: &ServiceCanisters,
    user_principal: Principal,
) -> Result<SessionType, String> {
    let result = query::<_, GetSessionTypeResult>(
        pocket_ic,
        service_canisters.user_info_service_canister_id,
        user_principal,
        "get_session_type_principal",
        (user_principal,),
    )
    .expect("Failed to call get_session_type_principal");
    
    match result {
        GetSessionTypeResult::Ok(session_type) => Ok(session_type),
        GetSessionTypeResult::Err(e) => Err(e),
    }
}