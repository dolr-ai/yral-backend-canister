use std::collections::HashMap;

use candid::{CandidType, Deserialize, Principal};

use crate::{access_control::UserAccessRole, common::types::known_principal::KnownPrincipalMap};

#[derive(Deserialize, CandidType, Default, Clone)]
pub struct UserIndexInitArgs {
    pub known_principal_ids: Option<KnownPrincipalMap>,
    pub access_control_map: Option<HashMap<Principal, Vec<UserAccessRole>>>,
    pub version: String,
}
