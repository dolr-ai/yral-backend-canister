use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult;
use shared_utils::canister_specific::individual_user_template::types::session::SessionType;

// User info service canister ID
pub const USER_INFO_SERVICE_CANISTER_ID: &str = "qhbym-qaaaa-aaaaa-aaafq-cai";

#[derive(CandidType, Deserialize, Debug)]
pub enum GetSessionTypeResult {
    Ok(SessionType),
    Err(String),
}

pub async fn get_session_type_for_principal(principal: Principal) -> Result<SessionType, String> {
    let canister_id = Principal::from_text(USER_INFO_SERVICE_CANISTER_ID)
        .expect("Invalid user info service canister ID");
    
    let result: CallResult<(GetSessionTypeResult,)> = ic_cdk::call(
        canister_id,
        "get_session_type_principal",
        (principal,),
    )
    .await;
    
    match result {
        Ok((GetSessionTypeResult::Ok(session_type),)) => Ok(session_type),
        Ok((GetSessionTypeResult::Err(e),)) => Err(format!("User info service error: {}", e)),
        Err((code, msg)) => Err(format!("Inter-canister call failed: {:?} - {}", code, msg)),
    }
}