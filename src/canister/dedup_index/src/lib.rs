use std::{
    cell::RefCell,
    collections::BTreeSet,
    ops::{Deref, DerefMut},
    time::SystemTime,
};

use candid::CandidType;
use ic_cdk::export_candid;
use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
};
use serde::{Deserialize, Serialize};

pub type Memory = VirtualMemory<DefaultMemoryImpl>;
type VideoId = String;
type VideoHash = String;

type Video = (VideoId, SystemTime);
#[derive(Clone, Debug, Serialize, Deserialize, CandidType, Default)]
struct Videos(pub BTreeSet<Video>);

impl Deref for Videos {
    type Target = BTreeSet<Video>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Videos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Storable for Videos {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut bytes = vec![];
        ciborium::ser::into_writer(self, &mut bytes).unwrap();

        std::borrow::Cow::Owned(bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        let value: Self = ciborium::de::from_reader(bytes.as_ref()).unwrap();
        value
    }
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static DEDUP_INDEX: RefCell<StableBTreeMap<VideoHash, Videos, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

#[ic_cdk_macros::update]
fn add_video_to_index(video_hash: String, video: Video) {
    // bar behind off-chain's caller id

    DEDUP_INDEX.with_borrow_mut(|index| {
        let Some(ref mut videos) = index.get(&video_hash) else {
            index.insert(video_hash, Videos([video].into()));
            return;
        };

        videos.insert(video);
    })
}

#[ic_cdk_macros::query]
fn is_duplicate(video_hash: String) -> bool {
    DEDUP_INDEX.with_borrow_mut(|index| index.contains_key(&video_hash))
}

export_candid!();
