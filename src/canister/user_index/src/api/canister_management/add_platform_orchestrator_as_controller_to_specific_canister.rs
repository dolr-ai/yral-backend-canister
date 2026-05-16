use candid::Principal;
use ic_cdk_macros::update;

use crate::util::canister_management::set_controller_with_platform_orchestrator;

const PLATFORM_ORCHESTRATOR_ID: &str = "74zq4-iqaaa-aaaam-ab53a-cai";

#[update]
pub async fn add_platform_orchestrator_as_controller_to_specific_canister(
    canister_id_being_updated: Principal,
) -> Result<(), String> {
    let platform_orchestrator =
        Principal::from_text(PLATFORM_ORCHESTRATOR_ID).map_err(|e| e.to_string())?;

    set_controller_with_platform_orchestrator(canister_id_being_updated, platform_orchestrator)
        .await
}
