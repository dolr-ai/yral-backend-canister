use std::{borrow::Cow, collections::BTreeSet, time::SystemTime};

use candid::{CandidType, Principal};
use ciborium::{de, ser};
use ic_cdk::println;
use ic_stable_structures::{BTreeMap as StableBTreeMap, Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::{
            profile::{
                UserProfile, UserProfileDetailsForFrontendV3, UserProfileDetailsForFrontendV4,
                UserProfileDetailsForFrontendV5, UserProfileDetailsForFrontendV6,
                UserProfileDetailsForFrontendV7,
            },
            session::SessionType,
        },
        user_info_service::types::{
            NSFWInfo, ProfilePictureData, ProfileUpdateDetails, ProfileUpdateDetailsV2, SubscriptionPlan,
        },
    },
    common::utils::system_time::get_current_system_time,
};
pub(crate) mod memory;

use crate::data_model::memory::Memory;

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub(crate) enum UserKind {
    User { bots: Vec<Principal> },
    Bot,
}

impl Default for UserKind {
    fn default() -> Self {
        UserKind::User { bots: Vec::new() }
    }
}

#[derive(CandidType, Deserialize, Serialize)]
pub(crate) struct UserInfo {
    profile: UserProfile,
    session_type: SessionType,
    last_access_time: SystemTime,
    #[serde(default)]
    followers: BTreeSet<Principal>,
    #[serde(default)]
    following: BTreeSet<Principal>,
    #[serde(default)]
    kind: UserKind,
}

impl UserInfo {
    pub fn new(user_principal: Principal) -> Self {
        Self {
            profile: UserProfile {
                principal_id: Some(user_principal),
                profile_stats: Default::default(),
                referrer_details: None,
                bio: None,
                website_url: None,
                subscription_plan: Default::default(),
                profile_picture: None,
                is_ai_influencer: false,
            },
            session_type: SessionType::AnonymousSession,
            last_access_time: get_current_system_time(),
            followers: BTreeSet::new(),
            following: BTreeSet::new(),
            kind: UserKind::User { bots: Vec::new() },
        }
    }

    pub fn authenticated(user_principal: Principal) -> Self {
        Self {
            profile: UserProfile {
                principal_id: Some(user_principal),
                profile_stats: Default::default(),
                referrer_details: None,
                bio: None,
                website_url: None,
                subscription_plan: Default::default(),
                profile_picture: None,
                is_ai_influencer: false,
            },
            session_type: SessionType::RegisteredSession,
            last_access_time: get_current_system_time(),
            followers: BTreeSet::new(),
            following: BTreeSet::new(),
            kind: UserKind::User { bots: Vec::new() },
        }
    }

    pub fn bot(bot_principal: Principal) -> Self {
        Self {
            profile: UserProfile {
                principal_id: Some(bot_principal),
                profile_stats: Default::default(),
                referrer_details: None,
                bio: None,
                website_url: None,
                subscription_plan: Default::default(),
                profile_picture: None,
                is_ai_influencer: false,
            },
            session_type: SessionType::RegisteredSession,
            last_access_time: get_current_system_time(),
            followers: BTreeSet::new(),
            following: BTreeSet::new(),
            kind: UserKind::Bot,
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
            println!("User already exists");
            return Ok(());
        }

        self.user_infos
            .insert(user_principal, UserInfo::new(user_principal));

        Ok(())
    }

    pub fn register_authenticated_user(
        &mut self,
        user_principal: Principal,
        authenticated: bool,
    ) -> Result<(), String> {
        if self.user_infos.contains_key(&user_principal) {
            println!("User already exists");
            return Ok(());
        }

        self.user_infos.insert(
            user_principal,
            if authenticated {
                UserInfo::authenticated(user_principal)
            } else {
                UserInfo::new(user_principal)
            },
        );

        Ok(())
    }

    pub fn register_authenticated_user_v2(
        &mut self,
        user_principal: Principal,
        authenticated: bool,
        bot_principal: Option<Principal>,
    ) -> Result<(), String> {
        if let Some(bot) = bot_principal {
            if self.user_infos.contains_key(&bot) {
                println!("Bot already exists");
                return Ok(());
            }

            let mut user_info = self
                .user_infos
                .get(&user_principal)
                .ok_or("Owner not found")?;

            match &mut user_info.kind {
                UserKind::User { bots } => {
                    // Register the bot
                    self.user_infos.insert(bot, UserInfo::bot(bot));
                    // Add to user's bots list
                    bots.push(bot);
                    self.user_infos.insert(user_principal, user_info);
                }
                UserKind::Bot => {
                    return Err("Bots cannot own other bots".to_string());
                }
            }
        } else {
            if self.user_infos.contains_key(&user_principal) {
                return Ok(());
            }

            self.user_infos.insert(
                user_principal,
                if authenticated {
                    UserInfo::authenticated(user_principal)
                } else {
                    UserInfo::new(user_principal)
                },
            );
        }

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
                profile_picture_url: user_info.profile.profile_picture.as_ref().map(|p| p.url.clone()),
            })
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn get_profile_details_for_user_v5(
        &self,
        user_principal: Principal,
        caller_principal: Principal,
    ) -> Result<UserProfileDetailsForFrontendV5, String> {
        if let Some(user_info) = self.user_infos.get(&user_principal) {
            Ok(UserProfileDetailsForFrontendV5 {
                principal_id: user_principal,
                bio: user_info.profile.bio.clone(),
                website_url: user_info.profile.website_url.clone(),
                followers_count: user_info.followers.len() as u64,
                following_count: user_info.following.len() as u64,
                caller_follows_user: user_info
                    .followers
                    .contains(&caller_principal)
                    .then_some(true)
                    .or_else(|| {
                        if caller_principal == user_principal {
                            None
                        } else {
                            Some(false)
                        }
                    }),
                user_follows_caller: user_info
                    .following
                    .contains(&caller_principal)
                    .then_some(true)
                    .or_else(|| {
                        if caller_principal == user_principal {
                            None
                        } else {
                            Some(false)
                        }
                    }),
                subscription_plan: user_info.profile.subscription_plan.clone(),
                profile_picture_url: user_info.profile.profile_picture.as_ref().map(|p| p.url.clone()),
            })
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn get_profile_details_for_user_v6(
        &self,
        user_principal: Principal,
        caller_principal: Principal,
    ) -> Result<UserProfileDetailsForFrontendV6, String> {
        if let Some(user_info) = self.user_infos.get(&user_principal) {
            Ok(UserProfileDetailsForFrontendV6 {
                principal_id: user_principal,
                profile_picture: user_info.profile.profile_picture.clone(),
                bio: user_info.profile.bio.clone(),
                website_url: user_info.profile.website_url.clone(),
                followers_count: user_info.followers.len() as u64,
                following_count: user_info.following.len() as u64,
                caller_follows_user: user_info
                    .followers
                    .contains(&caller_principal)
                    .then_some(true)
                    .or_else(|| {
                        if caller_principal == user_principal {
                            None
                        } else {
                            Some(false)
                        }
                    }),
                user_follows_caller: user_info
                    .following
                    .contains(&caller_principal)
                    .then_some(true)
                    .or_else(|| {
                        if caller_principal == user_principal {
                            None
                        } else {
                            Some(false)
                        }
                    }),
                subscription_plan: user_info.profile.subscription_plan.clone(),
                is_ai_influencer: user_info.profile.is_ai_influencer,
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
                _ => {
                    ic_cdk::println!(
                        "Session type of user {user_principal} can only be updated from AnonymousSession"
                    );
                    Ok(())
                }
            }
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn delete_user_info(&mut self, user_principal: Principal) -> Result<(), String> {
        // Get bots to delete if this is a user (not a bot)
        let bots_to_delete = self
            .user_infos
            .get(&user_principal)
            .and_then(|info| match &info.kind {
                UserKind::User { bots } => Some(bots.clone()),
                UserKind::Bot => None,
            })
            .unwrap_or_default();

        // Cascade delete all bots
        for bot_principal in bots_to_delete {
            self.user_infos.remove(&bot_principal);
        }

        // Then delete the user
        let removed_user = self.user_infos.remove(&user_principal);
        if removed_user.is_some() {
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn delete_bot_account(
        &mut self,
        bot_principal: Principal,
        caller: Principal,
    ) -> Result<(), String> {
        if !self.user_infos.contains_key(&bot_principal) {
            return Err("Bot not found".to_string());
        }

        let mut caller_info = self
            .user_infos
            .get(&caller)
            .ok_or("Caller not found".to_string())?;

        match &mut caller_info.kind {
            UserKind::User { bots } => {
                if !bots.contains(&bot_principal) {
                    return Err("Not authorized - only owner can delete bot".to_string());
                }
                bots.retain(|b| *b != bot_principal);
                self.user_infos.insert(caller, caller_info);
                self.user_infos.remove(&bot_principal);
                Ok(())
            }
            UserKind::Bot => Err("Bots cannot own other bots".to_string()),
        }
    }

    pub fn get_bots_by_owner(&self, owner: Principal) -> Vec<Principal> {
        self.user_infos
            .get(&owner)
            .and_then(|info| match &info.kind {
                UserKind::User { bots } => Some(bots.clone()),
                UserKind::Bot => None,
            })
            .unwrap_or_default()
    }

    pub fn follow_user(&mut self, follower: Principal, target: Principal) -> Result<(), String> {
        if follower == target {
            return Err("Cannot follow yourself".to_string());
        }

        let mut follower_info = self
            .user_infos
            .get(&follower)
            .ok_or("Follower not found".to_string())?;

        let mut target_info = self
            .user_infos
            .get(&target)
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

        let mut follower_info = self
            .user_infos
            .get(&follower)
            .ok_or("Follower not found".to_string())?;

        let mut target_info = self
            .user_infos
            .get(&target)
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
            return Err(format!(
                "Limit exceeds maximum of {}",
                MAX_FOLLOWERS_PER_PAGE
            ));
        }

        let user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        let mut followers_iter = match start {
            Some(cursor) => {
                // Use range to get all followers after the cursor (O(log n) operation)
                user_info.followers.range((
                    std::ops::Bound::Included(cursor),
                    std::ops::Bound::Unbounded,
                ))
            }
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
            return Err(format!(
                "Limit exceeds maximum of {}",
                MAX_FOLLOWING_PER_PAGE
            ));
        }

        let user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        let mut following_iter = match start {
            Some(cursor) => {
                // Use range to get all following after the cursor (O(log n) operation)
                user_info.following.range((
                    std::ops::Bound::Included(cursor),
                    std::ops::Bound::Unbounded,
                ))
            }
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
        let user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        Ok(user_info.followers.len() as u64)
    }

    pub fn get_following_count(&self, user_principal: Principal) -> Result<u64, String> {
        let user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        Ok(user_info.following.len() as u64)
    }

    pub fn update_profile_details(
        &mut self,
        user_principal: Principal,
        details: ProfileUpdateDetails,
    ) -> Result<(), String> {
        let mut user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        // Only update fields that have Some value
        if let Some(bio) = details.bio {
            user_info.profile.bio = Some(bio);
        }

        if let Some(website_url) = details.website_url {
            user_info.profile.website_url = Some(website_url);
        }

        if let Some(profile_picture_url) = details.profile_picture_url {
            // Update profile_picture with the new URL, keeping existing nsfw_info or defaulting to safe values
            let nsfw_info = user_info
                .profile
                .profile_picture
                .as_ref()
                .map(|p| p.nsfw_info.clone())
                .unwrap_or_default();
            user_info.profile.profile_picture = Some(ProfilePictureData {
                url: profile_picture_url,
                nsfw_info,
            });
        }

        self.user_infos.insert(user_principal, user_info);
        Ok(())
    }

    pub fn update_profile_details_v2(
        &mut self,
        user_principal: Principal,
        details: ProfileUpdateDetailsV2,
    ) -> Result<(), String> {
        let mut user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        // Only update fields that have Some value
        if let Some(bio) = details.bio {
            user_info.profile.bio = Some(bio);
        }

        if let Some(website_url) = details.website_url {
            user_info.profile.website_url = Some(website_url);
        }

        if let Some(profile_picture) = details.profile_picture {
            user_info.profile.profile_picture = Some(profile_picture);
        }

        self.user_infos.insert(user_principal, user_info);
        Ok(())
    }

    /// Admin-only method to update NSFW info for a user's profile picture
    pub fn update_profile_picture_nsfw_info(
        &mut self,
        user_principal: Principal,
        nsfw_info: NSFWInfo,
    ) -> Result<(), String> {
        let mut user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        if let Some(ref mut profile_picture) = user_info.profile.profile_picture {
            profile_picture.nsfw_info = nsfw_info;
        } else {
            return Err("User has no profile picture set".to_string());
        }

        self.user_infos.insert(user_principal, user_info);
        Ok(())
    }

    /// Admin-only method to update AI influencer status for a user's profile
    pub fn update_profile_ai_influencer_status(
        &mut self,
        user_principal: Principal,
        is_ai_influencer: bool,
    ) -> Result<(), String> {
        let mut user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        user_info.profile.is_ai_influencer = is_ai_influencer;

        self.user_infos.insert(user_principal, user_info);
        Ok(())
    }

    pub fn get_profile_details_for_user_v7(
        &self,
        user_principal: Principal,
        caller_principal: Principal,
    ) -> Result<UserProfileDetailsForFrontendV7, String> {
        if let Some(user_info) = self.user_infos.get(&user_principal) {
            Ok(UserProfileDetailsForFrontendV7 {
                principal_id: user_principal,
                profile_picture: user_info.profile.profile_picture.clone(),
                bio: user_info.profile.bio.clone(),
                website_url: user_info.profile.website_url.clone(),
                followers_count: user_info.followers.len() as u64,
                following_count: user_info.following.len() as u64,
                caller_follows_user: user_info
                    .followers
                    .contains(&caller_principal)
                    .then_some(true)
                    .or_else(|| {
                        if caller_principal == user_principal {
                            None
                        } else {
                            Some(false)
                        }
                    }),
                user_follows_caller: user_info
                    .following
                    .contains(&caller_principal)
                    .then_some(true)
                    .or_else(|| {
                        if caller_principal == user_principal {
                            None
                        } else {
                            Some(false)
                        }
                    }),
                subscription_plan: user_info.profile.subscription_plan.clone(),
                is_ai_influencer: user_info.profile.is_ai_influencer,
                bots: match &user_info.kind {
                    UserKind::User { bots } => bots.clone(),
                    UserKind::Bot => Vec::new(),
                },
            })
        } else {
            Err("User not found".to_string())
        }
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
                profile_picture_url: user_info.profile.profile_picture.as_ref().map(|p| p.url.clone()),
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
        include_profile_pics: bool,
    ) -> Result<Vec<shared_utils::canister_specific::user_info_service::types::FollowerItem>, String>
    {
        let items = follower_principals
            .into_iter()
            .map(|principal| {
                // Check if caller follows this follower
                let caller_follows = if caller_principal != Principal::anonymous() {
                    self.user_infos
                        .get(&caller_principal)
                        .map(|info| info.following.contains(&principal))
                        .unwrap_or(false)
                } else {
                    false
                };

                // Get profile picture if requested
                let profile_picture_url = if include_profile_pics {
                    self.user_infos
                        .get(&principal)
                        .and_then(|info| info.profile.profile_picture.as_ref().map(|p| p.url.clone()))
                } else {
                    None
                };

                shared_utils::canister_specific::user_info_service::types::FollowerItem {
                    principal_id: principal,
                    caller_follows,
                    profile_picture_url,
                }
            })
            .collect();

        Ok(items)
    }

    pub fn build_following_items(
        &self,
        caller_principal: Principal,
        following_principals: Vec<Principal>,
        include_profile_pics: bool,
    ) -> Result<Vec<shared_utils::canister_specific::user_info_service::types::FollowerItem>, String>
    {
        let items = following_principals
            .into_iter()
            .map(|principal| {
                // Check if caller follows this user (in following list)
                let caller_follows = if caller_principal != Principal::anonymous() {
                    self.user_infos
                        .get(&caller_principal)
                        .map(|info| info.following.contains(&principal))
                        .unwrap_or(false)
                } else {
                    false
                };

                // Get profile picture if requested
                let profile_picture_url = if include_profile_pics {
                    self.user_infos
                        .get(&principal)
                        .and_then(|info| info.profile.profile_picture.as_ref().map(|p| p.url.clone()))
                } else {
                    None
                };

                shared_utils::canister_specific::user_info_service::types::FollowerItem {
                    principal_id: principal,
                    caller_follows,
                    profile_picture_url,
                }
            })
            .collect();

        Ok(items)
    }

    pub fn change_subscription_plan(
        &mut self,
        user_principal: Principal,
        new_plan: SubscriptionPlan,
    ) -> Result<(), String> {
        let mut user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        user_info.profile.subscription_plan = new_plan;

        self.user_infos.insert(user_principal, user_info);
        Ok(())
    }

    pub fn remove_pro_plan_free_video_credits(
        &mut self,
        user_principal: Principal,
        credits_to_remove: u32,
    ) -> Result<(), String> {
        let mut user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        match &mut user_info.profile.subscription_plan {
            SubscriptionPlan::Pro(pro_subscription) => {
                if pro_subscription.free_video_credits_left < credits_to_remove {
                    return Err("Not enough free video credits".to_string());
                }
                pro_subscription.free_video_credits_left -= credits_to_remove;
                self.user_infos.insert(user_principal, user_info);
                Ok(())
            }
            SubscriptionPlan::Free => Err("User is on Free plan".to_string()),
        }
    }

    pub fn add_pro_plan_free_video_credits(
        &mut self,
        user_principal: Principal,
        credits_to_add: u32,
    ) -> Result<(), String> {
        let mut user_info = self
            .user_infos
            .get(&user_principal)
            .ok_or("User not found".to_string())?;

        match &mut user_info.profile.subscription_plan {
            SubscriptionPlan::Pro(pro_subscription) => {
                match pro_subscription
                    .free_video_credits_left
                    .checked_add(credits_to_add)
                {
                    Some(new_credits) => {
                        pro_subscription.free_video_credits_left = new_credits;
                        self.user_infos.insert(user_principal, user_info);
                        Ok(())
                    }
                    None => Err("Overflow when adding free credits".to_string()),
                }
            }
            SubscriptionPlan::Free => Err("User is on Free plan".to_string()),
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
