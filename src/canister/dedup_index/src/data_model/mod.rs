mod memory;
use ic_stable_structures::StableBTreeMap;
use memory::{Memory, get_dedup_index_memory};
use shared_utils::canister_specific::dedup_index::{VideoHash, Videos};

pub struct CanisterData {
    pub index: StableBTreeMap<VideoHash, Videos, Memory>,
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            index: StableBTreeMap::init(get_dedup_index_memory()),
        }
    }
}
