use candid::{CandidType, Deserialize, Principal};
use ic_cdk::export_candid;
use shared_utils::service::ServiceInitArgs;
use std::cell::RefCell;

pub mod api;
pub mod canister_lifecycle;
pub mod data_model;
pub mod utils;

pub use data_model::{CanisterData, GlobalRateLimitConfig, RateLimitConfig};

thread_local! {
    pub static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
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

export_candid!();
