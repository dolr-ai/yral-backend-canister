use candid::Principal;
use ic_cdk::api::management_canister::main::{start_canister, uninstall_code, CanisterIdRecord};
use ic_cdk_macros::update;
use shared_utils::common::utils::{
    permissions::is_caller_controller_or_global_admin,
    upgrade_canister::try_stopping_canister_with_retries,
};

use crate::{
    util::canister_management::check_and_request_cycles_from_platform_orchestrator, CANISTER_DATA,
};

#[update(guard = "is_caller_controller_or_global_admin")]
pub async fn uninstall_individual_user_canister(canister_id: Principal) -> Result<(), String> {
    let canister_id: Principal = CANISTER_DATA.with_borrow_mut(|canister_data| {
        let user_principal_to_remove = canister_data
            .user_principal_id_to_canister_id_map
            .iter()
            .find(|(_, v)| **v == canister_id)
            .map(|(k, _)| *k);

        let Some(user_principal_to_remove) = user_principal_to_remove else {
            return Err("Canister not found".to_string());
        };

        canister_data
            .user_principal_id_to_canister_id_map
            .remove(&user_principal_to_remove)
            .ok_or("Canister not found".to_string())
    })?;

    let _ = check_and_request_cycles_from_platform_orchestrator().await;

    try_stopping_canister_with_retries(canister_id, 3)
        .await
        .map_err(|e| format!("could not stop canister {:?} {}", e.0, e.1))?;

    uninstall_code(CanisterIdRecord { canister_id })
        .await
        .map_err(|e| {
            format!(
                "Failed to uninstall code for canister {}: {}",
                canister_id.to_text(),
                e.1
            )
        })?;

    start_canister(CanisterIdRecord { canister_id })
        .await
        .map_err(|e| {
            format!(
                "Failed to start canister {} after uninstalling code: {}",
                canister_id.to_text(),
                e.1
            )
        })?;

    CANISTER_DATA
        .with_borrow_mut(|canister_data| canister_data.backup_canister_pool.insert(canister_id));

    Ok(())
}
