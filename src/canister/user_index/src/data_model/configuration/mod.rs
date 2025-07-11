use candid::{CandidType, Deserialize};
use serde::Serialize;
use shared_utils::common::types::known_principal::KnownPrincipalMap;

#[derive(Deserialize, CandidType, Serialize, Clone)]
pub struct Configuration {
    pub known_principal_ids: KnownPrincipalMap,
    pub signups_open_on_this_subnet: bool,
    pub url_to_send_canister_metrics_to: String,
}
impl Default for Configuration {
    fn default() -> Self {
        Self { known_principal_ids: Default::default(), signups_open_on_this_subnet: true, url_to_send_canister_metrics_to: Default::default() }
    }
}
