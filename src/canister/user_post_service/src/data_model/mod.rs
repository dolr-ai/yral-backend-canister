pub mod memory;

use candid::Principal;
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl, StableBTreeMap};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::error::GetPostsOfUserProfileError,
        user_post_service::types::{
            args::PostDetailsForFrontend,
            args::PostDetailsFromFrontend,
            error::UserPostServiceError,
            storage::{Post, PostIdList},
        },
    },
    common::types::top_posts::post_score_index_item::PostStatus,
    pagination::{self, PaginationError},
};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type PostId = String;

#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    /// Map of post_id -> Post details
    #[serde(skip, default = "_init_posts")]
    posts: StableBTreeMap<PostId, Post, Memory>,

    /// Semantic version of this canister's code/data schema
    pub version: String,
}

impl CanisterData {
    pub fn add_post_to_memory(
        &mut self,
        post_from_frontend: PostDetailsFromFrontend,
    ) -> Result<(), UserPostServiceError> {
        let post = Post::from(post_from_frontend);

        if self.posts.contains_key(&post.id) {
            return Err(UserPostServiceError::DuplicatePostId);
        }

        self.posts.insert(post.id.clone(), post);

        Ok(())
    }

    pub fn add_post(&mut self, post: Post) -> Option<Post> {
        self.posts.insert(post.id.clone(), post)
    }

    pub fn get_post_details_for_user(
        &self,
        post_id: &PostId,
        user: Principal,
    ) -> Result<PostDetailsForFrontend, UserPostServiceError> {
        let post = self.get_post(post_id)?;
        Ok(post.get_post_details_for_frontend_for_user(user))
    }

    pub fn get_posts_of_this_user_profile_with_pagination_cursor(
        &self,
        creator: Principal,
        mut limit: usize,
        offset: usize,
    ) -> Vec<Post> {
        limit = limit.min(100);
        self.posts
            .iter()
            .filter(|(_, post)| {
                post.creator_principal == creator
                    && post.status != PostStatus::Deleted
                    && post.status != PostStatus::BannedDueToUserReporting
            })
            .map(|(_, post)| post)
            .skip(offset)
            .take(limit)
            .collect()
    }

    pub fn get_posts_of_this_user_profile_with_pagination(
        &self,
        creator: Principal,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Post>, GetPostsOfUserProfileError> {
        let posts_created_by_user: Vec<Post> = self
            .posts
            .iter()
            .filter(|(_, post)| {
                post.creator_principal == creator
                    && post.status != PostStatus::Deleted
                    && post.status != PostStatus::BannedDueToUserReporting
            })
            .map(|(_, post)| post)
            .collect();

        let (from_inclusive_index, limit) = pagination::get_pagination_bounds_cursor(
            offset as u64,
            limit as u64,
            posts_created_by_user.len() as u64,
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

        Ok(posts_created_by_user
            .into_iter()
            .skip(from_inclusive_index as usize)
            .take(limit as usize)
            .collect())
    }

    pub fn get_post(&self, post_id: &PostId) -> Result<Post, UserPostServiceError> {
        match self.posts.get(post_id) {
            Some(post) => match post.status {
                PostStatus::Deleted => Err(UserPostServiceError::PostNotFound),
                _ => Ok(post),
            },
            None => Err(UserPostServiceError::PostNotFound),
        }
    }

    pub fn delete_post(
        &mut self,
        post_id: &PostId,
        caller_principal: Principal,
    ) -> Result<(), UserPostServiceError> {
        let mut post = self
            .posts
            .get(&post_id)
            .ok_or(UserPostServiceError::PostNotFound)?;

        if post.creator_principal != caller_principal {
            return Err(UserPostServiceError::Unauthorized);
        }

        match post.status {
            PostStatus::Deleted => Err(UserPostServiceError::PostNotFound),
            _ => {
                post.status = PostStatus::Deleted;
                self.posts.insert(post_id.clone(), post);
                Ok(())
            }
        }
    }
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            posts: _init_posts(),
            version: String::from("v1.0.0"),
        }
    }
}

fn _init_posts() -> StableBTreeMap<PostId, Post, Memory> {
    StableBTreeMap::init(memory::get_posts_memory())
}

fn _init_posts_by_creator() -> StableBTreeMap<Principal, PostIdList, Memory> {
    StableBTreeMap::init(memory::get_posts_by_creator_memory())
}
