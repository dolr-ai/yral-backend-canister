use candid::CandidType;
use ic_cdk::api::call::RejectionCode;
use serde::Deserialize;

#[derive(CandidType, Deserialize, PartialEq, Eq, Debug)]
pub enum NotificationStoreError {
    Unauthorized,
    CallError(RejectionCode, String),
}

impl From<(RejectionCode, String)> for NotificationStoreError {
    fn from(value: (RejectionCode, String)) -> Self {
        NotificationStoreError::CallError(value.0, value.1)
    }
}
