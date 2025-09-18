use candid::Principal;

#[derive(candid::CandidType, candid::Deserialize, Debug, Clone, PartialEq)]
pub struct FollowerItem {
    pub principal_id: Principal,
    pub caller_follows: bool,
}

#[derive(candid::CandidType, candid::Deserialize, Debug, Clone, PartialEq)]
pub struct FollowingItem {
    pub principal_id: Principal,
    pub caller_follows: bool,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct FollowersResponse {
    pub followers: Vec<FollowerItem>,
    pub total_count: u64,
    pub next_cursor: Option<Principal>,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct FollowingResponse {
    pub following: Vec<FollowingItem>,
    pub total_count: u64,
    pub next_cursor: Option<Principal>,
}