use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::export_candid;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::Serialize;
use std::borrow::Cow;
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static VIDEO_METADATA_STORE: RefCell<StableBTreeMap<u64, VideoMetadata, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
struct VideoMetadata{
    pub user_principal: Principal,
    pub user_canister_id: Principal,
    pub is_landscape: bool,
    pub duration_secs: u64,
    // add more
}

impl BoundedStorable for VideoMetadata {
    const IS_FIXED_SIZE: bool = false;
    const MAX_SIZE: u32 = u32::MAX;
}

impl Storable for VideoMetadata {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
        
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

#[ic_cdk_macros::query]
fn get(key: u64) -> Option<VideoMetadata> {
    VIDEO_METADATA_STORE.with(|p| p.borrow().get(&key))
}

#[ic_cdk_macros::update]
fn insert(key: u64, value: VideoMetadata){
    VIDEO_METADATA_STORE.with(|p| p.borrow_mut().insert(key, value));
}

export_candid!();