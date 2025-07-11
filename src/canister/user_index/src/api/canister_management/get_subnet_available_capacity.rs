use ic_cdk_macros::query;
use std::borrow::Borrow;

use crate::CANISTER_DATA;

#[query]
pub fn get_subnet_available_capacity() -> u64 {
    CANISTER_DATA
        .with_borrow(|canister_data| canister_data.borrow().available_canisters.len() as u64)
}
