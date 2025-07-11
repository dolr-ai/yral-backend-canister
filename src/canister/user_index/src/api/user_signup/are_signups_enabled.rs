use ic_cdk_macros::query;

use crate::{CANISTER_DATA, data_model::CanisterData};


#[query]
fn are_signups_enabled() -> bool {
    CANISTER_DATA.with(|canister_data_ref_cell| {
        let canister_data = canister_data_ref_cell.borrow();
        are_signups_enabled_impl(&canister_data)
    })
}

fn are_signups_enabled_impl(canister_data: &CanisterData) -> bool {
    canister_data.configuration.signups_open_on_this_subnet
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::data_model::configuration::Configuration;

    use super::*;

    #[test]
    fn test_are_signups_enabled_impl() {
        let mut canister_data = CanisterData {
            configuration: Configuration {
                known_principal_ids: HashMap::default(),
                signups_open_on_this_subnet: true,
                url_to_send_canister_metrics_to: String::from("http://example.com")
            },
            ..Default::default()
        };

        assert!(are_signups_enabled_impl(&canister_data));

        canister_data.configuration.signups_open_on_this_subnet = false;
        assert!(!are_signups_enabled_impl(&canister_data));
    }
}
