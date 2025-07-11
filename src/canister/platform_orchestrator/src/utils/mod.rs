use candid::Principal;
use ic_cdk::api::management_canister::main::{
    deposit_cycles, CanisterIdRecord, CanisterInstallMode, InstallCodeArgument,
};
use shared_utils::{
    canister_specific::{
        platform_orchestrator::types::args::UpgradeCanisterArg,
        user_index::types::args::UserIndexInitArgs,
    },
    common::{types::wasm::WasmType, utils::upgrade_canister::upgrade_canister_util},
    constant::{
        SUBNET_ORCHESTRATOR_CANISTER_CYCLES_THRESHOLD, SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES,
    },
};

pub mod registered_subnet_orchestrator;

pub(crate) async fn recharge_and_upgrade_subnet_orchestrator(
    canister_id: Principal,
    upgrade_arg: UpgradeCanisterArg,
) -> Result<Principal, (Principal, String)> {
    match upgrade_arg.canister {
        WasmType::SubnetOrchestratorWasm => {
            recharge_subnet_orchestrator_if_needed(canister_id)
                .await
                .map_err(|e| (canister_id, e))?;
            upgrade_subnet_orchestrator_canister(
                canister_id,
                upgrade_arg.wasm_blob,
                upgrade_arg.version,
            )
            .await
            .map_err(|e| (canister_id, e))?;
        }
        _ => {}
    }
    Ok(canister_id)
}

pub(crate) async fn recharge_subnet_orchestrator_if_needed(
    canister_id: Principal,
) -> Result<(), String> {
    let (subnet_orchestrator_cycle_balance,): (u128,) =
        ic_cdk::call(canister_id, "get_user_index_canister_cycle_balance", ())
            .await
            .map_err(|e| e.1)?;

    if subnet_orchestrator_cycle_balance < SUBNET_ORCHESTRATOR_CANISTER_CYCLES_THRESHOLD {
        deposit_cycles(
            CanisterIdRecord { canister_id },
            SUBNET_ORCHESTRATOR_CANISTER_INITIAL_CYCLES,
        )
        .await
        .map_err(|e| e.1)?
    }
    Ok(())
}


pub(crate) async fn upgrade_subnet_orchestrator_canister(
    canister_id: Principal,
    wasm: Vec<u8>,
    version: String,
) -> Result<(), String> {
    let install_code_arg = InstallCodeArgument {
        mode: CanisterInstallMode::Upgrade(None),
        canister_id,
        wasm_module: wasm,
        arg: candid::encode_one(UserIndexInitArgs {
            known_principal_ids: None,
            access_control_map: None,
            version,
        })
        .unwrap(),
    };

    upgrade_canister_util(install_code_arg)
        .await
        .map_err(|e| e.1)
}
