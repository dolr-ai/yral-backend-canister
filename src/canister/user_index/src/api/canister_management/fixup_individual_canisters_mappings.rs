use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::utils::{permissions::is_caller_controller, task::run_task_concurrently};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller")]
pub async fn fixup_individual_canisters_mapping() {

    let user_canisters = CANISTER_DATA.with_borrow(|canister_data| canister_data.user_principal_id_to_canister_id_map.clone().into_iter());
    let user_canisters_futures = user_canisters.map(|(user_principal, user_canister)| async move {
        let result = ic_cdk::call::<_, (String,)>(user_canister, "get_version", ()).await;
        if let Err(e) = result {
            //IC0536 is the code for invalid method name. If the canister does not have get_version that means it has a wams installed that does not correspond to individual wasm.
            if e.1.contains("IC0536") {
                Err(user_principal)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    });

    let result_callback = |res: Result<(), Principal>| {
        if let Err(e) = res {
            CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data.user_principal_id_to_canister_id_map.remove(&e);
            })
        }
    };

    ic_cdk::spawn(run_task_concurrently(user_canisters_futures, 10, result_callback, || false));

    let available_canisters = CANISTER_DATA.with_borrow(|canister_data| {
        canister_data.available_canisters.clone()
    });
    


    let available_canisters_futures = available_canisters.into_iter().map(|available_canister|  async move {
        let result = ic_cdk::call::<_, (String,)>(available_canister, "get_version", ()).await;
        if let Err(e) = result {
            if e.1.contains("IC0536") {
                Err(available_canister)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    });


    
    let result_callback = |res: Result<(), Principal>| {
        if let Err(e) = res {
            CANISTER_DATA.with_borrow_mut(|canister_data| {
                canister_data.available_canisters.remove(&e);
            })
        }
    };

    ic_cdk::spawn(run_task_concurrently(available_canisters_futures, 10, result_callback, || false));
}