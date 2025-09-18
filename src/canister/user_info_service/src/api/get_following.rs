use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::query;

use crate::CANISTER_DATA;
use shared_utils::canister_specific::user_info_service::types::FollowingResponse;

#[query]
fn get_following(user: Principal, start: Option<Principal>, limit: u64) -> Result<FollowingResponse, String> {
    let caller_principal = caller();

    CANISTER_DATA.with_borrow(|canister_data| {
        let (following_principals, next_cursor) = canister_data.get_following_paginated(user, start, limit)?;
        let total_count = canister_data.get_following_count(user)?;

        // Check if caller follows each user in the following list
        let following = canister_data.build_following_items(caller_principal, following_principals)?;

        Ok(FollowingResponse {
            following,
            total_count,
            next_cursor,
        })
    })
}