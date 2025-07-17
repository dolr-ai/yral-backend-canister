pub mod memory;

use candid::Principal;
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl, StableBTreeMap,};
use serde::{Deserialize, Serialize};
use shared_utils::canister_specific::user_post_service::types::storage::{Post, PostIdList};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    /// Map of post_id -> Post details
    #[serde(skip, default = "_init_posts")]
    pub posts: StableBTreeMap<u64, Post, Memory>,

    /// Index: creator principal -> vector of post ids (most-recent first)
    #[serde(skip, default = "_init_posts_by_creator")]
    pub posts_by_creator: StableBTreeMap<Principal, PostIdList, Memory>,

    /// Counter to assign new unique post ids
    pub next_post_id: u64,

    /// Semantic version of this canister's code/data schema
    pub version: String,
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            posts: _init_posts(),
            posts_by_creator: _init_posts_by_creator(),
            next_post_id: 0,
            version: String::from("v1.0.0"),
        }
    }
}

fn _init_posts() -> StableBTreeMap<u64, Post, Memory> {
    StableBTreeMap::init(memory::get_posts_memory())
}

fn _init_posts_by_creator() -> StableBTreeMap<Principal, PostIdList, Memory> {
    StableBTreeMap::init(memory::get_posts_by_creator_memory())
}