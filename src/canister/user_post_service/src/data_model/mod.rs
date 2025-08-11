pub mod memory;

use candid::Principal;
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl, StableBTreeMap};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::user_post_service::types::{
        args::PostDetailsFromFrontend,
        error::UserPostServiceError,
        storage::{Post, PostIdList, VideoSourceType},
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
    pub fn update_playback_source(
        &mut self,
        post_id: &PostId,
        source_type: VideoSourceType,
        source_url: String,
    ) -> Result<(), String> {
        // TODO: validate url is valid
        let Some(mut post) = self.posts.get(post_id) else {
            return Err("Post not found".into());
        };

        post.playback_sources.insert(source_type, source_url);

        // update
        self.add_post(post);

        Ok(())
    }

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
