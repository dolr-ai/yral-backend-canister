use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult;
use shared_utils::canister_specific::individual_user_template::types::session::SessionType;

#[derive(CandidType, Deserialize, Debug)]
pub enum GetSessionTypeResult {
    Ok(SessionType),
    Err(String),
}

pub async fn get_session_type_for_principal(
    user_info_canister: Principal,
    principal: Principal,
) -> Result<SessionType, String> {
    let result: CallResult<(GetSessionTypeResult,)> = ic_cdk::call(
        user_info_canister,
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