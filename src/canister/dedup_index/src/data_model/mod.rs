pub mod memory;
use ic_stable_structures::StableBTreeMap;
use memory::{Memory, get_dedup_index_memory};
use serde::{Deserialize, Serialize};
use shared_utils::{
    canister_specific::dedup_index::{VideoHash, Videos},
    service::SetVersion,
};

#[derive(Serialize, Deserialize)]
pub struct CanisterData {
    #[serde(skip, default = "init_index")]
    pub index: StableBTreeMap<VideoHash, Videos, Memory>,
    pub version: String,
}

impl SetVersion for CanisterData {
    fn set_version(&mut self, version: &str) {
        self.version = version.into();
    }
}

fn init_index() -> StableBTreeMap<VideoHash, Videos, Memory> {
    StableBTreeMap::init(get_dedup_index_memory())
}

impl Default for CanisterData {
    fn default() -> Self {
        Self {
            index: init_index(),
            version: "v1.0.0".into(),
        }
    }
}
