use ic_cdk_macros::init;

use crate::{CANISTER_DATA};
use crate::types::args::UserPostServiceInitArgs;

#[init]
fn init(args: UserPostServiceInitArgs) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.version = args.version;
    });
}
