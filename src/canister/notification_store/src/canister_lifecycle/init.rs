use ic_cdk_macros::init;
use shared_utils::canister_specific::notification_store::types::args::NotificationStoreInitArgs;

use crate::{set_pruning_timer, CANISTER_DATA};

#[init]
fn init(args: NotificationStoreInitArgs) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.version = args.version;
    });
    set_pruning_timer();
}
