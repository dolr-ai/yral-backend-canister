use candid::Principal;
use ic_cdk_macros::update;
use shared_utils::common::types::known_principal::KnownPrincipalType;

use crate::{CANISTER_DATA, data_model::CanisterData};


#[update]
fn toggle_signups_enabled() -> Result<(), String> {
    let api_caller = ic_cdk::caller();

    CANISTER_DATA.with(|canister_data_ref_cell| {
        let mut canister_data = canister_data_ref_cell.borrow_mut();

        toggle_signups_enabled_impl(api_caller, &mut canister_data)
    })
}

fn toggle_signups_enabled_impl(
    caller: Principal,
    canister_data: &mut CanisterData,
) -> Result<(), String> {
    let super_admin = canister_data
        .configuration
        .known_principal_ids
        .get(&KnownPrincipalType::UserIdGlobalSuperAdmin)
        .ok_or("Super admin not found in internal records")?;

    if caller != *super_admin {
        return Err("Unauthorized".to_string());
    }

    canister_data.configuration.signups_open_on_this_subnet = !canister_data.configuration.signups_open_on_this_subnet;

    Ok(())
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id, get_mock_user_alice_principal_id,
    };


    use super::*;

    #[test]
    fn test_toggle_signups_enabled_impl() {
        let mut canister_data = CanisterData::default();
        canister_data.configuration.known_principal_ids.insert(
            KnownPrincipalType::UserIdGlobalSuperAdmin,
            get_global_super_admin_principal_id(),
        );

        let admin_caller = get_global_super_admin_principal_id();
        // super admin should be allowed to toggle
        let result = toggle_signups_enabled_impl(admin_caller, &mut canister_data);
        assert!(result.is_ok());
        assert!(!canister_data.configuration.signups_open_on_this_subnet);
        let result = toggle_signups_enabled_impl(admin_caller, &mut canister_data);
        assert!(result.is_ok());
        assert!(canister_data.configuration.signups_open_on_this_subnet);

        // non super admin should not be allowed to toggle
        let non_admin_caller = get_mock_user_alice_principal_id();
        let result = toggle_signups_enabled_impl(non_admin_caller, &mut canister_data);
        assert!(result.is_err());
        assert!(canister_data.configuration.signups_open_on_this_subnet);
    }
}
