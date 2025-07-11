use ic_cdk_macros::query;

use crate::CANISTER_DATA;

#[query]
fn get_version() -> String {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.version.clone())
}
