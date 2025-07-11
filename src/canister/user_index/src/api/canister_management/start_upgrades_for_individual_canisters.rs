use ic_cdk_macros::update;

use crate::{
    api::upgrade_individual_user_template::update_user_index_upgrade_user_canisters_with_latest_wasm,
    CANISTER_DATA,
};
use shared_utils::common::{
    types::wasm::{CanisterWasm, WasmType},
    utils::permissions::is_caller_controller,
};

#[update(guard = "is_caller_controller")]
async fn start_upgrades_for_individual_canisters(
    version: String,
    individual_user_wasm: Vec<u8>,
) -> String {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.allow_upgrades_for_individual_canisters = true;
        canister_data.last_run_upgrade_status.version = version.clone();
        let canister_wasm = CanisterWasm {
            version: version.clone(),
            wasm_blob: individual_user_wasm.clone(),
        };
        canister_data
            .wasms
            .insert(WasmType::IndividualUserWasm, canister_wasm);
    });
    ic_cdk::spawn(update_user_index_upgrade_user_canisters_with_latest_wasm::upgrade_user_canisters_with_latest_wasm(version, individual_user_wasm));
    "Success".to_string()
}
