use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::query;

use crate::CANISTER_DATA;
use shared_utils::canister_specific::user_info_service::types::FollowersResponse;

#[query]
fn get_followers(user: Principal, start: Option<Principal>, limit: u64) -> Result<FollowersResponse, String> {
    let caller_principal = caller();

    CANISTER_DATA.with_borrow(|canister_data| {
        let (follower_principals, next_cursor) = canister_data.get_followers_paginated(user, start, limit)?;
        let total_count = canister_data.get_followers_count(user)?;

        // Check if caller follows each follower
        let followers = canister_data.build_follower_items(caller_principal, follower_principals)?;

        Ok(FollowersResponse {
            followers,
            total_count,
            next_cursor,
        })
    })
}