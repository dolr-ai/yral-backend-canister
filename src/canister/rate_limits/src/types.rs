use candid::{CandidType, Deserialize, Principal};
use shared_utils::service::GetVersion;
use std::borrow::Cow;

#[derive(CandidType, Deserialize, Clone)]
pub struct RateLimitsInitArgs {
    pub version: String,
    pub user_info_canister: Principal,
}

impl GetVersion for RateLimitsInitArgs {
    fn get_version(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.version)
    }
}