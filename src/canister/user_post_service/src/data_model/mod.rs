pub mod memory;

use std::time::SystemTime;

use candid::Principal;
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl, StableBTreeMap};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::{
        individual_user_template::types::error::GetPostsOfUserProfileError,
        user_post_service::types::{
            args::PostDetailsForFrontend, args::PostDetailsFromFrontend,
            error::UserPostServiceError, storage::Post,
        },
    },
    common::types::top_posts::post_score_index_item::PostStatus,
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
        let posts: Vec<Post> = self
            .posts
            .iter()
            .filter(|(_, post)| {
                post.creator_principal == creator
                    && post.status != PostStatus::Deleted
                    && post.status != PostStatus::BannedDueToUserReporting
            })
            .map(|(_, post)| post)
            .collect();

        posts.into_iter().skip(offset).take(limit).collect()
    }

    pub fn get_posts_of_this_user_profile_with_pagination(
        &self,
        creator: Principal,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Post>, GetPostsOfUserProfileError> {
        if limit == 0 {
            return Err(GetPostsOfUserProfileError::InvalidBoundsPassed);
        }

        let max_items_needed = offset + limit;
        let mut posts_created_by_user: Vec<Post> = Vec::with_capacity(max_items_needed);

        // Collect posts until we have enough (offset + limit) or exhaust all posts
        for (_, post) in self.posts.iter() {
            if post.creator_principal == creator
                && post.status != PostStatus::Deleted
                && post.status != PostStatus::BannedDueToUserReporting
            {
                posts_created_by_user.push(post);

                // Stop early if we have collected enough posts for pagination
                if posts_created_by_user.len() >= max_items_needed {
                    break;
                }
            }
        }

        // Check if we have enough posts for the requested offset
        if offset >= posts_created_by_user.len() {
            return Err(GetPostsOfUserProfileError::ReachedEndOfItemsList);
        }

        posts_created_by_user.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Return the requested slice
        let end_index = std::cmp::min(offset + limit, posts_created_by_user.len());
        Ok(posts_created_by_user[offset..end_index].to_vec())
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
