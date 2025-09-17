use std::{borrow::Cow, collections::BTreeSet, time::SystemTime};

use candid::{CandidType, Principal};
use ciborium::{de, ser};
use ic_stable_structures::{BTreeMap as StableBTreeMap, Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::individual_user_template::types::{
        profile::{UserProfile, UserProfileDetailsForFrontendV3, UserProfileDetailsForFrontendV4},
        session::SessionType,
    },
    common::utils::system_time::get_current_system_time,
};
pub(crate) mod memory;

use crate::data_model::memory::Memory;

#[derive(CandidType, Deserialize)]
pub struct ProfileUpdateDetails {
    pub bio: Option<String>,
    pub website_url: Option<String>,
    pub profile_picture_url: Option<String>,
}

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
                bio: None,
                website_url: None,
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
        start: Option<Principal>,
        limit: u64,
    ) -> Result<(Vec<Principal>, Option<Principal>), String> {
        const MAX_FOLLOWERS_PER_PAGE: u64 = 100;

        if limit > MAX_FOLLOWERS_PER_PAGE {
            return Err(format!("Limit exceeds maximum of {}", MAX_FOLLOWERS_PER_PAGE));
        }

        let user_info = self.user_infos.get(&user_principal)
            .ok_or("User not found".to_string())?;

        let mut followers_iter = match start {
            Some(cursor) => {
                // Use range to get all followers after the cursor (O(log n) operation)
                user_info.followers.range((std::ops::Bound::Included(cursor), std::ops::Bound::Unbounded))
            },
            None => {
                // Start from the beginning
                user_info.followers.range(..)
            }
        };

        let followers: Vec<Principal> = followers_iter
            .by_ref()
            .take(limit as usize)
            .cloned()
            .collect();

        // Get the next cursor if there are more items
        let next_cursor = followers_iter.next().cloned();

        Ok((followers, next_cursor))
    }

    pub fn get_following_paginated(
        &self,
        user_principal: Principal,
        start: Option<Principal>,
        limit: u64,
    ) -> Result<(Vec<Principal>, Option<Principal>), String> {
        const MAX_FOLLOWING_PER_PAGE: u64 = 100;

        if limit > MAX_FOLLOWING_PER_PAGE {
            return Err(format!("Limit exceeds maximum of {}", MAX_FOLLOWING_PER_PAGE));
        }

        let user_info = self.user_infos.get(&user_principal)
            .ok_or("User not found".to_string())?;

        let mut following_iter = match start {
            Some(cursor) => {
                // Use range to get all following after the cursor (O(log n) operation)
                user_info.following.range((std::ops::Bound::Included(cursor), std::ops::Bound::Unbounded))
            },
            None => {
                // Start from the beginning
                user_info.following.range(..)
            }
        };

        let following: Vec<Principal> = following_iter
            .by_ref()
            .take(limit as usize)
            .cloned()
            .collect();

        // Get the next cursor if there are more items
        let next_cursor = following_iter.next().cloned();

        Ok((following, next_cursor))
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

    pub fn update_profile_details(
        &mut self,
        user_principal: Principal,
        details: ProfileUpdateDetails,
    ) -> Result<(), String> {
        let mut user_info = self.user_infos.get(&user_principal)
            .ok_or("User not found".to_string())?;

        // Only update fields that have Some value
        if let Some(bio) = details.bio {
            user_info.profile.bio = Some(bio);
        }
        
        if let Some(website_url) = details.website_url {
            user_info.profile.website_url = Some(website_url);
        }
        
        if let Some(profile_picture_url) = details.profile_picture_url {
            user_info.profile.profile_picture_url = Some(profile_picture_url);
        }

        self.user_infos.insert(user_principal, user_info);
        Ok(())
    }

    pub fn get_profile_details_v4(
        &self,
        caller_principal: Principal,
        user_principal: Principal,
    ) -> Result<UserProfileDetailsForFrontendV4, String> {
        if let Some(user_info) = self.user_infos.get(&user_principal) {
            // Determine the follow relationships
            let (caller_follows_user, user_follows_caller) = if caller_principal == user_principal {
                // Can't follow yourself
                (None, None)
            } else {
                // Check if caller is in the user's followers set
                let caller_follows = user_info.followers.contains(&caller_principal);
                // Check if user is in the caller's followers set (i.e., caller is in user's following set)
                let user_follows = user_info.following.contains(&caller_principal);
                (Some(caller_follows), Some(user_follows))
            };

            Ok(UserProfileDetailsForFrontendV4 {
                principal_id: user_principal,
                profile_stats: user_info.profile.profile_stats,
                profile_picture_url: user_info.profile.profile_picture_url.clone(),
                bio: user_info.profile.bio.clone(),
                website_url: user_info.profile.website_url.clone(),
                followers_count: user_info.followers.len() as u64,
                following_count: user_info.following.len() as u64,
                caller_follows_user,
                user_follows_caller,
            })
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn build_follower_items(
        &self,
        caller_principal: Principal,
        follower_principals: Vec<Principal>,
    ) -> Result<Vec<shared_utils::canister_specific::user_info_service::types::FollowerItem>, String> {
        let items = follower_principals.into_iter().map(|principal| {
            // Check if caller follows this follower
            let caller_follows = if caller_principal != Principal::anonymous() {
                self.user_infos.get(&caller_principal)
                    .map(|info| info.following.contains(&principal))
                    .unwrap_or(false)
            } else {
                false
            };

            shared_utils::canister_specific::user_info_service::types::FollowerItem {
                principal_id: principal,
                caller_follows,
            }
        }).collect();

        Ok(items)
    }

    pub fn build_following_items(
        &self,
        caller_principal: Principal,
        following_principals: Vec<Principal>,
    ) -> Result<Vec<shared_utils::canister_specific::user_info_service::types::FollowingItem>, String> {
        let items = following_principals.into_iter().map(|principal| {
            // Check if caller follows this user (in following list)
            let caller_follows = if caller_principal != Principal::anonymous() {
                self.user_infos.get(&caller_principal)
                    .map(|info| info.following.contains(&principal))
                    .unwrap_or(false)
            } else {
                false
            };

            shared_utils::canister_specific::user_info_service::types::FollowingItem {
                principal_id: principal,
                caller_follows,
            }
        }).collect();

        Ok(items)
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
