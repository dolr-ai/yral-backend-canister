use ic_stable_structures::{DefaultMemoryImpl, memory_manager::{self, MemoryId, MemoryManager, VirtualMemory}};
use std::cell::RefCell;

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const UPGRADES: MemoryId = MemoryId::new(0);

// A memory for BtreeMap to store wasms
const SUBNET_ORCHESTRATOR_WASM_MEMORY: MemoryId = MemoryId::new(1);

//A memory for canister upgrade log index
const CANISTER_UPGRADE_LOG_INDEX: MemoryId = MemoryId::new(2);


//A memory for canister upgrade log 
const CANISTER_UPGRADE_LOG: MemoryId = MemoryId::new(3);

// A memory for the StableBTreeMap we're using. A new memory should be created for
// every additional stable structure.


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

pub fn get_subnet_orchestrator_wasm_memory() -> Memory {
    MEMORY_MANAGER.with_borrow_mut(|memory_manager| memory_manager.get(SUBNET_ORCHESTRATOR_WASM_MEMORY))
}

pub fn get_canister_upgrade_log_index_memory() -> Memory {
    MEMORY_MANAGER.with_borrow_mut(|memory_manager| memory_manager.get(CANISTER_UPGRADE_LOG_INDEX))
}

pub fn get_canister_upgrade_log_memory() -> Memory {
    MEMORY_MANAGER.with_borrow_mut(|memory_manager| memory_manager.get(CANISTER_UPGRADE_LOG))
}

pub fn init_memory_manager() {
    MEMORY_MANAGER.with(|m| {
        *m.borrow_mut() = MemoryManager::init_with_bucket_size(DefaultMemoryImpl::default(), 1);
    })
}
