use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize)]
pub struct RateLimitsInitArgs {
    pub version: String,
    pub user_info_canister: Principal,
}