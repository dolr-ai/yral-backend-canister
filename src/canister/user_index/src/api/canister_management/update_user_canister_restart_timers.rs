use futures::StreamExt;
use ic_cdk::api::call::CallResult;
use ic_cdk_macros::update;

use crate::CANISTER_DATA;
use shared_utils::common::utils::permissions::is_caller_controller;

#[update(guard = "is_caller_controller")]
async fn update_restart_timers_hon_game() -> String {
    ic_cdk::spawn(update_restart_timers_hon_game_impl());
    "Success".to_string()
}

async fn update_restart_timers_hon_game_impl() {
    let canisters = CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data
            .user_principal_id_to_canister_id_map
            .values()
            .cloned()
            .collect::<Vec<_>>()
    });

    let futures = canisters.iter().map(|canister_id| async {
        let _: CallResult<()> = ic_cdk::call(
            *canister_id,
            "once_reenqueue_timers_for_pending_bet_outcomes",
            (),
        )
        .await;
    });

    let stream = futures::stream::iter(futures).boxed().buffer_unordered(25);

    let _ = stream.collect::<Vec<()>>().await;
}
