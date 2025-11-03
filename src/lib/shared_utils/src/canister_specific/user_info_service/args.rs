use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType)]
pub struct UserInfoServiceInitArgs {
    pub version: String,
}
