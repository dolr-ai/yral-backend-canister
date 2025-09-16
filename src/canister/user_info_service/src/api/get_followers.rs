use candid::Principal;
use ic_cdk_macros::query;

use crate::CANISTER_DATA;
use shared_utils::canister_specific::user_info_service::types::FollowersResponse;

#[query]
fn get_followers(user: Principal, offset: u64, limit: u64) -> Result<FollowersResponse, String> {
    CANISTER_DATA.with_borrow(|canister_data| {
        let followers = canister_data.get_followers_paginated(user, offset, limit)?;
        let total_count = canister_data.get_followers_count(user)?;
        
        Ok(FollowersResponse {
            followers,
            total_count,
        })
    })
}