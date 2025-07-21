use candid::CandidType;
use ic_cdk::api::call::RejectionCode;
use serde::Deserialize;

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum UserPostServiceError {
    Unauthorized,
    PostNotFound,
    CallError(RejectionCode, String),
    DuplicatePostId,
}

impl From<(RejectionCode, String)> for UserPostServiceError {
    fn from(value: (RejectionCode, String)) -> Self {
        UserPostServiceError::CallError(value.0, value.1)
    }
}
