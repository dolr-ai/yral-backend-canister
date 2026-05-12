use ic_cdk::query;
use ic_cdk_macros::export_candid;

#[query]
fn hello_world() -> String {
    "Hello from individual user canister!".to_string()
}

export_candid!();
