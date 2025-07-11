use candid::Principal;
use ic_cdk_macros::query;
use shared_utils::common::types::known_principal::{KnownPrincipalMap, KnownPrincipalType};

use crate::CANISTER_DATA;

#[query]
fn get_current_list_of_all_well_known_principal_values() -> Vec<(KnownPrincipalType, Principal)> {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let known_principal_ids = &canister_data_ref_cell.borrow().configuration.known_principal_ids;
        get_current_list_of_all_well_known_principal_values_impl(known_principal_ids)
    })
}

fn get_current_list_of_all_well_known_principal_values_impl(
    known_principal_ids: &KnownPrincipalMap,
) -> Vec<(KnownPrincipalType, Principal)> {
    known_principal_ids
        .iter()
        .map(|(known_principal_type, principal)| (*known_principal_type, *principal))
        .collect()
}

#[cfg(test)]
mod test {
    use test_utils::setup::test_constants::{
        get_global_super_admin_principal_id,
        get_mock_canister_id_user_index,
    };

    use super::*;

    #[test]
    fn test_get_well_known_principal_value_impl() {
        let mut known_principal_ids = KnownPrincipalMap::new();
        known_principal_ids.insert(
            KnownPrincipalType::UserIdGlobalSuperAdmin,
            get_global_super_admin_principal_id(),
        );
        known_principal_ids.insert(
            KnownPrincipalType::CanisterIdUserIndex,
            get_mock_canister_id_user_index(),
        );

        assert!(
            get_current_list_of_all_well_known_principal_values_impl(&known_principal_ids)
                .contains(&(
                    KnownPrincipalType::UserIdGlobalSuperAdmin,
                    get_global_super_admin_principal_id()
                ))
        );
        assert!(
            get_current_list_of_all_well_known_principal_values_impl(&known_principal_ids)
                .contains(&(
                    KnownPrincipalType::CanisterIdUserIndex,
                    get_mock_canister_id_user_index()
                ))
        );
    }
}
