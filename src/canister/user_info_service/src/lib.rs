use std::cell::RefCell;

use ic_cdk::export_candid;

use crate::data_model::CanisterData;

mod api;
pub mod data_model;

thread_local! {
    static CANISTER_DATA: RefCell<CanisterData> = RefCell::default();
}

export_candid!();
