use candid::Principal;

#[derive(candid::CandidType, candid::Deserialize)]
pub struct FollowersResponse {
    pub followers: Vec<Principal>,
    pub total_count: u64,
}

#[derive(candid::CandidType, candid::Deserialize)]
pub struct FollowingResponse {
    pub following: Vec<Principal>,
    pub total_count: u64,
}