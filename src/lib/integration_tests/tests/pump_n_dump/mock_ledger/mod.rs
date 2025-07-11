pub mod mock_ledger_intf;

use candid::{Nat, Principal};
use mock_ledger_intf::{Account, FeatureFlags, InitArgs, InitArgsArchiveOptions, LedgerArg};
use pocket_ic::PocketIc;
use test_utils::setup::test_constants::v1::CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS;

const ICRC1_LEDGER_WASM: &[u8] = include_bytes!("ic-icrc1-ledger.wasm.gz");
pub const LEDGER_FEE: u64 = 10_000;
pub const LEDGER_MINT_AMOUNT: u64 = 10_000_000_000;

pub fn deploy(pic: &PocketIc, owner: Principal) -> Principal {
    let icrc1 = pic.create_canister();
    pic.add_cycles(
        icrc1,
        CANISTER_INITIAL_CYCLES_FOR_SPAWNING_CANISTERS
    );

    let init_args = InitArgs {
        token_symbol: "DOLLR".into(),
        token_name: "DOLLR".into(),
        decimals: None,
        transfer_fee: Nat::from(LEDGER_FEE),
        metadata: vec![],
        minting_account: Account {
            owner,
            subaccount: None,
        },
        initial_balances: vec![
            (
                Account {
                    owner,
                    subaccount: None,
                },
                Nat::from(LEDGER_MINT_AMOUNT),
            )
        ],
        maximum_number_of_accounts: None,
        accounts_overflow_trim_quantity: None,
        fee_collector_account: None,
        max_memo_length: None,
        feature_flags: Some(FeatureFlags {
            icrc2: true,
        }),
        archive_options: InitArgsArchiveOptions {
            num_blocks_to_archive: 1000,
            max_transactions_per_response: None,
            trigger_threshold: 2000,
            more_controller_ids: None,
            max_message_size_bytes: None,
            cycles_for_archive_creation: Some(10000000000000u64),
            node_max_memory_size_bytes: None,
            controller_id: owner,
        },
    };
    let ledger_arg = LedgerArg::Init(init_args);
    let ledger_args_raw = candid::encode_one(ledger_arg).unwrap();

    pic.install_canister(
        icrc1,
        ICRC1_LEDGER_WASM.into(),
        ledger_args_raw,
        None
    );

    icrc1
}