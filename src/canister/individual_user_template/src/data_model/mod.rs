use std::{
    collections::{BTreeMap, HashSet},
    time::SystemTime,
};

use candid::{Deserialize, Principal};
use memory::get_token_list_memory;
use serde::Serialize;
use shared_utils::{
    canister_specific::individual_user_template::types::{
        cdao::DeployedCdaoCanisters,
        error::GetPostsOfUserProfileError,
        migration::MigrationInfo,
        post::{Post, PostDetailsForFrontend, PostDetailsFromFrontend},
        profile::{UserProfile, UserProfileDetailsForFrontend},
        session::SessionType,
    },
    common::types::{
        known_principal::KnownPrincipalMap, top_posts::post_score_index_item::PostStatus,
        version_details::VersionDetails,
    },
    pagination::{self, PaginationError},
};

use self::memory::Memory;

pub mod memory;
pub mod pump_n_dump;

#[derive(Deserialize, Serialize)]
pub(crate) struct CanisterData {
    // Key is Post ID
    all_created_posts: BTreeMap<u64, Post>,
    pub known_principal_ids: KnownPrincipalMap,
    pub profile: UserProfile,
    pub version_details: VersionDetails,
    #[serde(default)]
    pub session_type: Option<SessionType>,
    #[serde(default)]
    pub last_access_time: Option<SystemTime>,
    #[serde(default)]
    pub migration_info: MigrationInfo,
    #[serde(default)]
    pub cdao_canisters: Vec<DeployedCdaoCanisters>,
    // list of root token canisters
    #[serde(skip, default = "_default_token_list")]
    pub token_roots: ic_stable_structures::btreemap::BTreeMap<Principal, (), Memory>,
    #[serde(default)]
    pub empty_canisters: AllotedEmptyCanister,
}

impl CanisterData {
    pub(crate) fn delete_post(&mut self, post_id: u64) -> Result<(), String> {
        let post = self
            .all_created_posts
            .get_mut(&post_id)
            .ok_or("Post not found".to_owned())?;

        match post.status {
            PostStatus::Deleted => Err("Post not found".to_owned()),
            _ => {
                post.status = PostStatus::Deleted;
                Ok(())
            }
        }
    }

    pub fn set_all_created_posts(&mut self, all_created_post: BTreeMap<u64, Post>) {
        self.all_created_posts = all_created_post;
    }

    pub fn get_all_posts_cloned(&self) -> Vec<(u64, Post)> {
        self.all_created_posts
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    pub fn add_post(&mut self, post: Post) -> Option<Post> {
        self.all_created_posts.insert(post.id, post)
    }

    pub fn contains_post(&self, post_id: &u64) -> bool {
        self.all_created_posts.contains_key(post_id)
    }

    pub fn add_post_to_memory(
        &mut self,
        post_details_from_frontend: &PostDetailsFromFrontend,
        current_time: &SystemTime,
    ) -> u64 {
        let post_id = self.all_created_posts.len() as u64;
        self.add_post(Post::new(post_id, post_details_from_frontend, current_time));

        post_id
    }

    pub fn get_posts_with_pagination_cursor(
        &self,
        from_inclusive_index: u64,
        limit: u64,
        api_caller_principal_id: Principal,
        current_time: SystemTime,
    ) -> Result<Vec<PostDetailsForFrontend>, GetPostsOfUserProfileError> {
        let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
            from_inclusive_index,
            limit,
            self.all_created_posts.len() as u64,
        )
        .map_err(|e| match e {
            PaginationError::InvalidBoundsPassed => GetPostsOfUserProfileError::InvalidBoundsPassed,
            PaginationError::ReachedEndOfItemsList => {
                GetPostsOfUserProfileError::ReachedEndOfItemsList
            }
            PaginationError::ExceededMaxNumberOfItemsAllowedInOneRequest => {
                GetPostsOfUserProfileError::ExceededMaxNumberOfItemsAllowedInOneRequest
            }
        })?;

        let res_posts = self
            .all_created_posts
            .iter()
            .filter(|(_, post)| {
                post.status != PostStatus::BannedDueToUserReporting
                    && post.status != PostStatus::Deleted
            })
            .rev()
            .skip(from_inclusive_index as usize)
            .take(limit as usize)
            .map(|(id, post)| {
                let profile = &self.profile;

                post.get_post_details_for_frontend_for_this_post(
                    UserProfileDetailsForFrontend {
                        display_name: None,
                        followers_count: 0,
                        following_count: 0,
                        principal_id: profile.principal_id.unwrap(),
                        profile_picture_url: profile.profile_picture_url.clone(),
                        profile_stats: profile.profile_stats,
                        unique_user_name: None,
                        lifetime_earnings: 0,
                        referrer_details: profile.referrer_details.clone(),
                    },
                    api_caller_principal_id,
                )
            })
            .collect();

        Ok(res_posts)
    }

    fn get_post_mut(posts: &mut BTreeMap<u64, Post>, post_id: u64) -> Option<&mut Post> {
        posts.get_mut(&post_id).and_then(|post| match post.status {
            PostStatus::Deleted => None,
            _ => Some(post),
        })
    }

    pub fn get_post(&self, post_id: &u64) -> Option<&Post> {
        self.all_created_posts
            .get(post_id)
            .and_then(|post| match post.status {
                PostStatus::Deleted => None,
                _ => Some(post),
            })
    }

    pub(crate) fn get_post_for_frontend(
        &self,
        post_id: u64,
        caller: Principal,
    ) -> PostDetailsForFrontend {
        let post = self.get_post(&post_id).unwrap();
        let profile = &self.profile;

        post.get_post_details_for_frontend_for_this_post(
            UserProfileDetailsForFrontend {
                display_name: None,
                followers_count: 0,
                following_count: 0,
                principal_id: profile.principal_id.unwrap(),
                profile_picture_url: profile.profile_picture_url.clone(),
                profile_stats: profile.profile_stats,
                unique_user_name: None,
                lifetime_earnings: 0,
                referrer_details: profile.referrer_details.clone(),
            },
            caller,
        )
    }

    pub(crate) fn get_total_number_of_posts(&self) -> u64 {
        self.all_created_posts
            .iter()
            .filter(|(_, post)| post.status != PostStatus::Deleted)
            .count() as u64
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct AllotedEmptyCanister {
    canister_ids: HashSet<Principal>,
}

impl AllotedEmptyCanister {
    pub fn get_number_of_canister(&mut self, number: usize) -> Result<Vec<Principal>, String> {
        let mut canister_ids = vec![];
        let mut iterator = self.canister_ids.iter().copied();
        for _ in 0..number {
            if let Some(canister_id) = iterator.next() {
                canister_ids.push(canister_id);
            } else {
                return Err(format!("{} number of canisters not available", number));
            }
        }

        self.canister_ids = iterator.collect();

        Ok(canister_ids)
    }

    pub fn get_empty_canisters(&self) -> Vec<Principal> {
        self.canister_ids.iter().copied().collect()
    }

    pub fn insert_empty_canister(&mut self, canister_id: Principal) -> bool {
        self.canister_ids.insert(canister_id)
    }

    pub fn append_empty_canisters(&mut self, canister_ids: Vec<Principal>) {
        self.canister_ids.extend(canister_ids.into_iter());
    }

    pub fn len(&self) -> usize {
        self.canister_ids.len()
    }
}

pub fn _default_token_list() -> ic_stable_structures::btreemap::BTreeMap<Principal, (), Memory> {
    ic_stable_structures::btreemap::BTreeMap::init(get_token_list_memory())
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            all_created_posts: BTreeMap::new(),
            known_principal_ids: KnownPrincipalMap::default(),
            profile: UserProfile::default(),
            version_details: VersionDetails::default(),
            session_type: None,
            last_access_time: None,
            migration_info: MigrationInfo::NotMigrated,
            cdao_canisters: Vec::new(),
            token_roots: _default_token_list(),
            empty_canisters: AllotedEmptyCanister::default(),
        }
    }
}
