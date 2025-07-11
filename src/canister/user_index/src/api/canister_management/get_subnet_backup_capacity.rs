use ic_cdk_macros::query;
use std::borrow::Borrow;

use crate::CANISTER_DATA;

#[query]
pub fn get_subnet_backup_capacity() -> u64 {
    CANISTER_DATA
        .with_borrow(|canister_data| canister_data.borrow().backup_canister_pool.len() as u64)
}
