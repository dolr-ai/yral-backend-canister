pub mod canister_lifecycle;
pub mod data_model;
pub mod api;

use std::cell::RefCell;

// Bring commonly used types into scope so that `export_candid!()` can locate them.
use candid::Principal;

// Types exposed in public Candid interface
use shared_utils::canister_specific::user_post_service::types::{args::UserPostServiceInitArgs, error::UserPostServiceError};
use shared_utils::canister_specific::user_post_service::types::storage::Post;

use crate::data_model::CanisterData;

thread_local! {
    /// Global state for this canister. All business-logic APIs must access state through this.
    pub static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

// -------- Candid export --------
use ic_cdk_macros::export_candid;
export_candid!();

