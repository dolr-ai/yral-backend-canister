use ic_cdk_macros::query;

use crate::{
    data_model::bulk_individual_canister_operation_status::BulkIndividualCanisterOperationStatus,
    CANISTER_DATA,
};

#[query]
pub fn get_bulk_operation_status() -> BulkIndividualCanisterOperationStatus {
    CANISTER_DATA.with_borrow(|canister_data| canister_data.bulk_operation_status.clone())
}
