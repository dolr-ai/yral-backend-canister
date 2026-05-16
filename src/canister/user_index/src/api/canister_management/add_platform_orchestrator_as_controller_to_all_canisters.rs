use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::task::run_task_concurrently;

use crate::{util::canister_management::set_controller_with_platform_orchestrator, CANISTER_DATA};

const PLATFORM_ORCHESTRATOR_ID: &str = "74zq4-iqaaa-aaaam-ab53a-cai";
const MAX_CONCURRENCY: usize = 10;

#[update]
pub fn add_platform_orchestrator_as_controller_to_all_canisters() -> Result<String, String> {
    let platform_orchestrator =
        Principal::from_text(PLATFORM_ORCHESTRATOR_ID).map_err(|e| e.to_string())?;

    let canister_ids: Vec<Principal> = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .user_principal_id_to_canister_id_map
            .values()
            .copied()
            .chain(canister_data.available_canisters.iter().copied())
            .chain(canister_data.backup_canister_pool.iter().copied())
            .collect()
    });

    CANISTER_DATA.with_borrow_mut(|canister_data| {
        let status = &mut canister_data.bulk_operation_status;
        status.canisters_remaining = canister_ids.iter().copied().collect();
        status.completed_count = 0;
        status.failed_canisters = Vec::new();
    });

    ic_cdk::spawn(async move {
        let futures = canister_ids
            .into_iter()
            .map(|canister_id_being_updated| async move {
                let res = set_controller_with_platform_orchestrator(
                    canister_id_being_updated,
                    platform_orchestrator,
                )
                .await;
                (canister_id_being_updated, res)
            });

        let result_callback =
            |(canister_id_being_updated, res): (Principal, Result<(), String>)| {
                CANISTER_DATA.with_borrow_mut(|canister_data| {
                    let status = &mut canister_data.bulk_operation_status;
                    status.canisters_remaining.remove(&canister_id_being_updated);
                    match res {
                        Ok(()) => status.completed_count += 1,
                        Err(reason) => status
                            .failed_canisters
                            .push((canister_id_being_updated, reason)),
                    }
                });
            };

        run_task_concurrently(futures, MAX_CONCURRENCY, result_callback, || false).await;
    });

    Ok("Started".to_string())
}
