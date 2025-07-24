use ic_stable_structures::{
    DefaultMemoryImpl,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};
use std::cell::RefCell;

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const UPGRADES: MemoryId = MemoryId::new(0);

// Renaming memory segments to be specific for post service and adding an additional segment for the creator index.
const POSTS_MEMORY: MemoryId = MemoryId::new(1);
const POSTS_BY_CREATOR_MEMORY: MemoryId = MemoryId::new(2);

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn get_upgrades_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(UPGRADES))
}

pub fn get_posts_memory() -> Memory {
    MEMORY_MANAGER.with_borrow_mut(|m| m.get(POSTS_MEMORY))
}

pub fn get_posts_by_creator_memory() -> Memory {
    MEMORY_MANAGER.with_borrow_mut(|m| m.get(POSTS_BY_CREATOR_MEMORY))
}

pub fn init_memory_manager() {
    MEMORY_MANAGER.with(|m| {
        *m.borrow_mut() = MemoryManager::init_with_bucket_size(DefaultMemoryImpl::default(), 2);
    })
}