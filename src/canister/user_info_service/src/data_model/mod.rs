use std::{borrow::Cow, collections::BTreeSet, time::SystemTime};

use candid::{CandidType, Principal};
use ciborium::{de, ser};
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
    followers: BTreeSet<Principal>,
    following: BTreeSet<Principal>,
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
            followers: BTreeSet::new(),
            following: BTreeSet::new(),
        }
    }
}

impl Storable for UserInfo {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ser::into_writer(self, &mut bytes).unwrap();
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        de::from_reader(bytes.as_ref()).unwrap()
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

    pub fn update_session_type_for_user(
        &mut self,
        user_principal: Principal,
        session_type: SessionType,
    ) -> Result<(), String> {
        if let Some(mut user_info) = self.user_infos.get(&user_principal) {
            match user_info.session_type {
                SessionType::AnonymousSession => {
                    user_info.session_type = session_type;
                    self.user_infos.insert(user_principal, user_info);
                    Ok(())
                }
                _ => Err("Session type can only be updated from AnonymousSession".to_string()),
            }
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn delete_user_info(&mut self, user_principal: Principal) -> Result<(), String> {
        let removed_user = self.user_infos.remove(&user_principal);
        if removed_user.is_some() {
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn follow_user(&mut self, follower: Principal, target: Principal) -> Result<(), String> {
        if follower == target {
            return Err("Cannot follow yourself".to_string());
        }

        let mut follower_info = self.user_infos.get(&follower)
            .ok_or("Follower not found".to_string())?;
        
        let mut target_info = self.user_infos.get(&target)
            .ok_or("Target user not found".to_string())?;

        if follower_info.following.contains(&target) {
            return Err("Already following this user".to_string());
        }

        follower_info.following.insert(target);
        target_info.followers.insert(follower);

        self.user_infos.insert(follower, follower_info);
        self.user_infos.insert(target, target_info);

        Ok(())
    }

    pub fn unfollow_user(&mut self, follower: Principal, target: Principal) -> Result<(), String> {
        if follower == target {
            return Err("Cannot unfollow yourself".to_string());
        }

        let mut follower_info = self.user_infos.get(&follower)
            .ok_or("Follower not found".to_string())?;
        
        let mut target_info = self.user_infos.get(&target)
            .ok_or("Target user not found".to_string())?;

        if !follower_info.following.contains(&target) {
            return Err("Not following this user".to_string());
        }

        follower_info.following.remove(&target);
        target_info.followers.remove(&follower);

        self.user_infos.insert(follower, follower_info);
        self.user_infos.insert(target, target_info);

        Ok(())
    }

    pub fn get_followers_paginated(
        &self,
        user_principal: Principal,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Principal>, String> {
        const MAX_FOLLOWERS_PER_PAGE: u64 = 100;
        
        if limit > MAX_FOLLOWERS_PER_PAGE {
            return Err(format!("Limit exceeds maximum of {}", MAX_FOLLOWERS_PER_PAGE));
        }

        let user_info = self.user_infos.get(&user_principal)
            .ok_or("User not found".to_string())?;

        let followers: Vec<Principal> = user_info.followers
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();

        Ok(followers)
    }

    pub fn get_following_paginated(
        &self,
        user_principal: Principal,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Principal>, String> {
        const MAX_FOLLOWING_PER_PAGE: u64 = 100;
        
        if limit > MAX_FOLLOWING_PER_PAGE {
            return Err(format!("Limit exceeds maximum of {}", MAX_FOLLOWING_PER_PAGE));
        }

        let user_info = self.user_infos.get(&user_principal)
            .ok_or("User not found".to_string())?;

        let following: Vec<Principal> = user_info.following
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();

        Ok(following)
    }

    pub fn get_followers_count(&self, user_principal: Principal) -> Result<u64, String> {
        let user_info = self.user_infos.get(&user_principal)
            .ok_or("User not found".to_string())?;
        
        Ok(user_info.followers.len() as u64)
    }

    pub fn get_following_count(&self, user_principal: Principal) -> Result<u64, String> {
        let user_info = self.user_infos.get(&user_principal)
            .ok_or("User not found".to_string())?;
        
        Ok(user_info.following.len() as u64)
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
