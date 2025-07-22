use candid::Principal;
use ic_cdk::export_candid;
use std::cell::RefCell;

pub mod api;
pub mod canister_lifecycle;
pub mod data_model;
pub mod types;

pub use data_model::CanisterData;
pub use shared_utils::canister_specific::rate_limits::{
    GlobalRateLimitConfig, PropertyRateLimitConfig, RateLimitConfig, RateLimitResult,
    RateLimitStatus, RateLimitsInitArgs,
};

thread_local! {
    pub static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();
