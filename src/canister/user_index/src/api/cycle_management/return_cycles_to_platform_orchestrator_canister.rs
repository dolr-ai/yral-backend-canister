use ic_cdk::api::{canister_balance128, management_canister::{main::deposit_cycles, provisional::CanisterIdRecord}};
use ic_cdk_macros::update;
use shared_utils::{common::{types::known_principal::KnownPrincipalType, utils::permissions::is_caller_controller}, constant::SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller")]
async fn return_cycles_to_platform_orchestrator_canister() -> Result<String, String> {
    let reclaim_amount = canister_balance128() - SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES;

    if reclaim_amount > 0 {

        let platform_orchestrator = CANISTER_DATA.with_borrow(|canister_data|
             *canister_data.configuration.known_principal_ids.get(&KnownPrincipalType::CanisterIdPlatformOrchestrator).unwrap()
            );

        deposit_cycles(
            CanisterIdRecord {
                canister_id: platform_orchestrator
            }, 
            reclaim_amount
        )
        .await
        .map_err(|_| String::from("Failed to deposit cycles to platform orchestrator"))?;
    }

    Ok("Success".into())
}