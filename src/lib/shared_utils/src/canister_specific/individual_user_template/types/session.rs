use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, CandidType, Debug, PartialEq)]
pub enum SessionType {
    AnonymousSession,
    RegisteredSession,
}