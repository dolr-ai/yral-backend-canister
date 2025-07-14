use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::{
    canister_specific::individual_user_template::types::session::SessionType,
    common::utils::permissions::is_caller_controller_or_global_admin,
};

use crate::CANISTER_DATA;

#[update(guard = "is_caller_controller_or_global_admin")]
fn update_session_type(user_principal: Principal, session_type: SessionType) -> Result<(), String> {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.update_session_type_for_user(user_principal, session_type)
    })
}
