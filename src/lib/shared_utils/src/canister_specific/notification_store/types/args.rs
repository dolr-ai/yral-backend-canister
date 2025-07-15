use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Clone)]
pub struct NotificationStoreInitArgs {
    pub version: String,
}