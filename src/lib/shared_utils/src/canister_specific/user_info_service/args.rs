use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType)]
pub struct UserInfoServiceInitArgs {
    pub version: String,
}

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct PostIdVideoUidMappingPaginationResult {
    pub result: Vec<(String, String)>,
    pub last_uuid_processed: Option<String>,
}
