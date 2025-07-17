use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize)]
pub struct UserPostServiceInitArgs {
    pub version: String,
} 