use ic_cdk_macros::init;
use shared_utils::canister_specific::user_info_service::args::UserInfoServiceInitArgs;

use crate::CANISTER_DATA;

#[init]
fn init(args: UserInfoServiceInitArgs) {
    CANISTER_DATA.with_borrow_mut(|canister_data| {
        canister_data.version = args.version.clone();
    })
}
