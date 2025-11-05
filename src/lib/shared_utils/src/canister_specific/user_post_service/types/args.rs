use std::time::SystemTime;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::{
    canister_specific::user_post_service::types::storage::Post,
    common::{
        types::top_posts::post_score_index_item::PostStatus,
        utils::system_time::get_current_system_time,
    },
};

#[derive(CandidType, Deserialize)]
pub struct UserPostServiceInitArgs {
    pub version: String,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct PostDetailsFromFrontend {
    pub id: String,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub creator_principal: Principal,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct PostDetailsFromFrontendV1 {
    pub id: String,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub creator_principal: Principal,
    pub status: PostStatusFromFrontend,
    pub created_at: SystemTime,
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub enum PostStatusFromFrontend {
    Draft,
    Published,
}

impl From<PostStatusFromFrontend> for PostStatus {
    fn from(status: PostStatusFromFrontend) -> Self {
        match status {
            PostStatusFromFrontend::Draft => PostStatus::Draft,
            PostStatusFromFrontend::Published => PostStatus::Uploaded,
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Serialize, Debug)]
pub struct PostDetailsForFrontend {
    pub id: String,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub creator_principal: Principal,
    pub created_at: SystemTime,
    pub total_view_count: u64,
    pub like_count: u64,
    pub created_by_user_principal_id: Principal,
    pub liked_by_me: bool,
}

impl From<PostDetailsFromFrontend> for Post {
    fn from(details: PostDetailsFromFrontend) -> Self {
        Self {
            description: details.description,
            hashtags: details.hashtags,
            video_uid: details.video_uid,
            creator_principal: details.creator_principal,
            status: Default::default(), // Default status
            created_at: get_current_system_time(),
            likes: Default::default(),
            share_count: 0,
            id: details.id,
            view_stats: Default::default(),
        }
    }
}

impl From<PostDetailsFromFrontendV1> for Post {
    fn from(details: PostDetailsFromFrontendV1) -> Self {
        Self {
            description: details.description,
            hashtags: details.hashtags,
            video_uid: details.video_uid,
            creator_principal: details.creator_principal,
            status: details.status.into(),
            created_at: details.created_at,
            likes: Default::default(),
            share_count: 0,
            id: details.id,
            view_stats: Default::default(),
        }
    }
}
