use std::{borrow::Cow, time::SystemTime};

use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{BTreeMap as StableBTreeMap, Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        profile::{UserProfile, UserProfileDetailsForFrontendV3},
        session::SessionType,
    },
    common::utils::system_time::get_current_system_time,
};
pub(crate) mod memory;

use crate::data_model::memory::Memory;

#[derive(CandidType, Deserialize, Serialize)]
pub(crate) struct UserInfo {
    profile: UserProfile,
    session_type: SessionType,
    last_access_time: SystemTime,
}

impl UserInfo {
    pub fn new(user_principal: Principal) -> Self {
        Self {
            profile: UserProfile {
                principal_id: Some(user_principal),
                profile_picture_url: None,
                profile_stats: Default::default(),
                referrer_details: None,
            },
            session_type: SessionType::AnonymousSession,
            last_access_time: get_current_system_time(),
        }
    }
}

impl Storable for UserInfo {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CanisterData {
    pub version: String, //semver version
    #[serde(skip, default = "_init_user_infos")]
    user_infos: StableBTreeMap<Principal, UserInfo, Memory>,
}

impl CanisterData {
    pub fn register_new_user(&mut self, user_principal: Principal) -> Result<(), String> {
        if self.user_infos.contains_key(&user_principal) {
            return Err("User already exists".to_string());
        }

        self.user_infos
            .insert(user_principal, UserInfo::new(user_principal));

        Ok(())
    }

    pub fn get_session_type_for_user(
        &self,
        user_principal: Principal,
    ) -> Result<SessionType, String> {
        if let Some(user_info) = self.user_infos.get(&user_principal) {
            Ok(user_info.session_type)
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn get_profile_details_for_user(
        &self,
        user_principal: Principal,
    ) -> Result<UserProfileDetailsForFrontendV3, String> {
        if let Some(user_info) = self.user_infos.get(&user_principal) {
            Ok(UserProfileDetailsForFrontendV3 {
                principal_id: user_principal,
                profile_stats: user_info.profile.profile_stats,
                profile_picture_url: user_info.profile.profile_picture_url.clone(),
            })
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn update_last_access_time_for_user(
        &mut self,
        user_principal: Principal,
    ) -> Result<(), String> {
        if let Some(mut user_info) = self.user_infos.get(&user_principal) {
            user_info.last_access_time = get_current_system_time();
            self.user_infos.insert(user_principal, user_info);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            version: String::from("v1.0.0"),
            user_infos: _init_user_infos(),
        }
    }
}

fn _init_user_infos() -> StableBTreeMap<Principal, UserInfo, Memory> {
    StableBTreeMap::init(memory::get_user_info_memory())
}
