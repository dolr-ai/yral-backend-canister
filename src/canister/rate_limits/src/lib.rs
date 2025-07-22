use candid::Principal;
use ic_cdk::export_candid;
use std::cell::RefCell;

pub mod api;
pub mod canister_lifecycle;
pub mod data_model;
pub mod types;
pub mod utils;

pub use data_model::{CanisterData, GlobalRateLimitConfig, PropertyRateLimitConfig};
pub use shared_utils::canister_specific::rate_limits::{
    RateLimitConfig, RateLimitResult, RateLimitStatus, RateLimitsInitArgs,
    GlobalRateLimitConfig as SharedGlobalRateLimitConfig,
    PropertyRateLimitConfig as SharedPropertyRateLimitConfig,
};

thread_local! {
    pub static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();
