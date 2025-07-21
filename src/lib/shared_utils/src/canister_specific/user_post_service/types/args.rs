use candid::{CandidType, Principal};
use serde::Deserialize;

use crate::canister_specific::user_post_service::types::storage::Post;

#[derive(CandidType, Deserialize)]
pub struct UserPostServiceInitArgs {
    pub version: String,
}

#[derive(CandidType, Deserialize)]
pub struct PostDetailsFromFrontend {
    pub id: String,
    pub description: String,
    pub hashtags: Vec<String>,
    pub video_uid: String,
    pub creator_principal: Principal,
}

impl From<PostDetailsFromFrontend> for Post {
    fn from(details: PostDetailsFromFrontend) -> Self {
        Self {
            description: details.description,
            hashtags: details.hashtags,
            video_uid: details.video_uid,
            creator_principal: details.creator_principal,
            status: Default::default(), // Default status
            created_at: std::time::SystemTime::now(),
            likes: Default::default(),
            share_count: 0,
            id: details.id,
            view_stats: Default::default(),
        }
    }
}
