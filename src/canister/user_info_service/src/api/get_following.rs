use candid::Principal;
use ic_cdk_macros::query;

use crate::CANISTER_DATA;

#[derive(candid::CandidType, candid::Deserialize)]
pub struct FollowingResponse {
    pub following: Vec<Principal>,
    pub total_count: u64,
}

#[query]
fn get_following(user: Principal, offset: u64, limit: u64) -> Result<FollowingResponse, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let following = canister_data.get_following_paginated(user, offset, limit)?;
        let total_count = canister_data.get_following_count(user)?;
        
        Ok(FollowingResponse {
            following,
            total_count,
        })
    })
}