use ic_stable_structures::{
    DefaultMemoryImpl,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};
use std::cell::RefCell;

const RATE_LIMITS: MemoryId = MemoryId::new(0);
const UPGRADES: MemoryId = MemoryId::new(1);
const PROPERTY_CONFIGS: MemoryId = MemoryId::new(2);
const PROPERTY_RATE_LIMITS: MemoryId = MemoryId::new(3);

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

pub fn get_rate_limits_memory() -> Memory {
    MEMORY_MANAGER.with_borrow_mut(|m| m.get(RATE_LIMITS))
}

pub fn get_upgrades_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(UPGRADES))
}

pub fn get_property_configs_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(PROPERTY_CONFIGS))
}

pub fn get_property_rate_limits_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow_mut().get(PROPERTY_RATE_LIMITS))
}