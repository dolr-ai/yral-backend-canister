use candid::Principal;
use ic_cdk::call;
use ic_cdk_macros::{query, update};
use shared_utils::common::{
    types::known_principal::KnownPrincipalType, utils::task::run_task_concurrently,
};

use crate::{guard::is_caller::is_caller_platform_global_admin_or_controller, CANISTER_DATA};

#[query]
fn get_global_known_principal(known_principal_type: KnownPrincipalType) -> Principal {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .known_principals
            .get_global_known_principal(&known_principal_type)
    })
}

#[query]
fn get_subnet_known_principal(
    subnet_orchestrator_id: Principal,
    known_principal_type: KnownPrincipalType,
) -> Principal {
    CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .known_principals
            .get_subnet_known_principal(&subnet_orchestrator_id, &known_principal_type)
    })
}

#[update(guard = "is_caller_platform_global_admin_or_controller")]
fn update_global_known_principal(
    known_principal_type: KnownPrincipalType,
    value: Principal,
) -> Result<String, String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .known_principals
            .add_global_known_principal(known_principal_type, value)
    });

    ic_cdk::spawn(issue_update_known_principal_for_all_subnet(
        known_principal_type,
        value,
    ));

    Ok("Success".into())
}

#[update(guard = "is_caller_platform_global_admin_or_controller")]
async fn update_subnet_known_principal(
    subnet_id: Principal,
    know_principal_type: KnownPrincipalType,
    value: Principal,
) -> Result<String, String> {
    call::<_, ()>(
        subnet_id,
        "update_well_known_principal",
        (know_principal_type.clone(), value),
    )
    .await
    .map_err(|e| e.1)?;
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .known_principals
            .add_subnet_orchestrator_known_principal(subnet_id, know_principal_type, value)
    });

    Ok("Success".into())
}

async fn issue_update_known_principal_for_all_subnet(
    known_principal_type: KnownPrincipalType,
    value: Principal,
) {
    let subnet_list: Vec<Principal> = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data
            .all_subnet_orchestrator_canisters_list
            .iter()
            .copied()
            .collect()
    });

    let futures = subnet_list.iter().map(|subnet_id| async {
        let res = call::<_, ()>(
            *subnet_id,
            "update_well_known_principal",
            (known_principal_type, value),
        )
        .await;
        match res {
            Ok(()) => Ok((*subnet_id, known_principal_type, value)),
            Err(e) => Err(e.1),
        }
    });

    let result_callback = |res| {
        if let Ok((subnet_id, known_principal_type, value)) = res {
            CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data
                    .known_principals
                    .add_subnet_orchestrator_known_principal(subnet_id, known_principal_type, value)
            });
        }
    };

    run_task_concurrently(futures, 10, result_callback, || false).await;
}
