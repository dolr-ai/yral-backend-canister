use ic_stable_structures::{
    DefaultMemoryImpl,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};
use std::cell::RefCell;

// reserving lower 16 slots for future use
const INDEX: MemoryId = MemoryId::new(0x10);

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn get_dedup_index_memory() -> Memory {
    MEMORY_MANAGER.with_borrow_mut(|m| m.get(INDEX))
}
