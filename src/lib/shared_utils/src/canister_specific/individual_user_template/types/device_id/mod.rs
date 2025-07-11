use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub timestamp: u64,
}
