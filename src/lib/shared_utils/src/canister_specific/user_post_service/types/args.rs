use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::{
    canister_specific::user_post_service::types::storage::Post,
    common::utils::system_time::get_current_system_time,
};

use super::storage::VideoSourceType;

#[derive(CandidType, Deserialize)]
pub struct UserPostServiceInitArgs {
    pub version: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct PostDetailsFromFrontend {
    pub id: String,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_url: String,
    pub creator_principal: Principal,
}

impl From<PostDetailsFromFrontend> for Post {
    fn from(details: PostDetailsFromFrontend) -> Self {
        Self {
            description: details.description,
            hashtags: details.hashtags,
            creator_principal: details.creator_principal,
            status: Default::default(), // Default status
            created_at: get_current_system_time(),
            likes: Default::default(),
            share_count: 0,
            id: details.id,
            view_stats: Default::default(),
            playback_sources: [(VideoSourceType::Raw, details.video_url)].into(),
        }
    }
}
