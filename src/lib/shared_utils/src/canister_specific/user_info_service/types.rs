use candid::Principal;

#[derive(candid::CandidType, candid::Deserialize, Debug, Clone, PartialEq)]
pub struct FollowerItem {
    pub principal_id: Principal,
    pub caller_follows: bool,
    pub profile_picture_url: Option<String>,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct FollowersResponse {
    pub followers: Vec<FollowerItem>,
    pub total_count: u64,
    pub next_cursor: Option<Principal>,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct FollowingResponse {
    pub following: Vec<FollowerItem>,
    pub total_count: u64,
    pub next_cursor: Option<Principal>,
}