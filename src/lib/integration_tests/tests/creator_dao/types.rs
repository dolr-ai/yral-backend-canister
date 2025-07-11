// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Nat, Principal};
use ic_cdk::api::call::CallResult as Result;
use serde_bytes;

#[derive(CandidType, Deserialize)]
pub struct Account {
  pub owner: Principal,
  pub subaccount: Option<serde_bytes::ByteBuf>,
}

#[derive(CandidType, Deserialize)]
pub struct FeatureFlags { pub icrc2: bool }

#[derive(CandidType, Deserialize)]
pub struct UpgradeArgs {
  pub icrc1_minting_account: Option<Account>,
  pub feature_flags: Option<FeatureFlags>,
}

#[derive(CandidType, Deserialize)]
pub struct Tokens { pub e8s: u64 }

#[derive(CandidType, Deserialize)]
pub struct Duration { pub secs: u64, pub nanos: u32 }

#[derive(CandidType, Deserialize)]
pub struct ArchiveOptions {
  pub num_blocks_to_archive: u64,
  pub max_transactions_per_response: Option<u64>,
  pub trigger_threshold: u64,
  pub more_controller_ids: Option<Vec<Principal>>,
  pub max_message_size_bytes: Option<u64>,
  pub cycles_for_archive_creation: Option<u64>,
  pub node_max_memory_size_bytes: Option<u64>,
  pub controller_id: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct InitArgs {
  pub send_whitelist: Vec<Principal>,
  pub token_symbol: Option<String>,
  pub transfer_fee: Option<Tokens>,
  pub minting_account: String,
  pub maximum_number_of_accounts: Option<u64>,
  pub accounts_overflow_trim_quantity: Option<u64>,
  pub transaction_window: Option<Duration>,
  pub max_message_size_bytes: Option<u64>,
  pub icrc1_minting_account: Option<Account>,
  pub archive_options: Option<ArchiveOptions>,
  pub initial_values: Vec<(String,Tokens,)>,
  pub token_name: Option<String>,
  pub feature_flags: Option<FeatureFlags>,
}

#[derive(CandidType, Deserialize)]
pub enum LedgerCanisterPayload { Upgrade(Option<UpgradeArgs>), Init(InitArgs) }

#[derive(CandidType, Deserialize)]
pub struct BinaryAccountBalanceArgs { pub account: serde_bytes::ByteBuf }

#[derive(CandidType, Deserialize)]
pub struct AccountBalanceArgs { pub account: String }

#[derive(CandidType, Deserialize)]
pub struct ArchiveInfo { pub canister_id: Principal }

#[derive(CandidType, Deserialize)]
pub struct Archives { pub archives: Vec<ArchiveInfo> }

#[derive(CandidType, Deserialize)]
pub struct Decimals { pub decimals: u32 }

#[derive(CandidType, Deserialize)]
pub struct StandardRecord { pub url: String, pub name: String }

#[derive(CandidType, Deserialize)]
pub enum MetadataValue {
  Int(candid::Int),
  Nat(candid::Nat),
  Blob(serde_bytes::ByteBuf),
  Text(String),
}

#[derive(CandidType, Deserialize)]
pub struct TransferArg {
  pub to: Account,
  pub fee: Option<candid::Nat>,
  pub memo: Option<serde_bytes::ByteBuf>,
  pub from_subaccount: Option<serde_bytes::ByteBuf>,
  pub created_at_time: Option<u64>,
  pub amount: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub enum TransferError {
  GenericError{ message: String, error_code: candid::Nat },
  TemporarilyUnavailable,
  BadBurn{ min_burn_amount: candid::Nat },
  Duplicate{ duplicate_of: candid::Nat },
  BadFee{ expected_fee: candid::Nat },
  CreatedInFuture{ ledger_time: u64 },
  TooOld,
  InsufficientFunds{ balance: candid::Nat },
}

#[derive(CandidType, Deserialize)]
pub enum Result_ { Ok(candid::Nat), Err(TransferError) }

#[derive(CandidType, Deserialize)]
pub struct ConsentMessageMetadata {
  pub utc_offset_minutes: Option<i16>,
  pub language: String,
}

#[derive(CandidType, Deserialize)]
pub enum DisplayMessageType {
  GenericDisplay,
  LineDisplay{ characters_per_line: u16, lines_per_page: u16 },
}

#[derive(CandidType, Deserialize)]
pub struct ConsentMessageSpec {
  pub metadata: ConsentMessageMetadata,
  pub device_spec: Option<DisplayMessageType>,
}

#[derive(CandidType, Deserialize)]
pub struct ConsentMessageRequest {
  pub arg: serde_bytes::ByteBuf,
  pub method: String,
  pub user_preferences: ConsentMessageSpec,
}

#[derive(CandidType, Deserialize)]
pub struct LineDisplayPage { pub lines: Vec<String> }

#[derive(CandidType, Deserialize)]
pub enum ConsentMessage {
  LineDisplayMessage{ pages: Vec<LineDisplayPage> },
  GenericDisplayMessage(String),
}

#[derive(CandidType, Deserialize)]
pub struct ConsentInfo {
  pub metadata: ConsentMessageMetadata,
  pub consent_message: ConsentMessage,
}

#[derive(CandidType, Deserialize)]
pub struct ErrorInfo { pub description: String }

#[derive(CandidType, Deserialize)]
pub enum Icrc21Error {
  GenericError{ description: String, error_code: candid::Nat },
  InsufficientPayment(ErrorInfo),
  UnsupportedCanisterCall(ErrorInfo),
  ConsentMessageUnavailable(ErrorInfo),
}

#[derive(CandidType, Deserialize)]
pub enum Result1 { Ok(ConsentInfo), Err(Icrc21Error) }

#[derive(CandidType, Deserialize)]
pub struct AllowanceArgs { pub account: Account, pub spender: Account }

#[derive(CandidType, Deserialize)]
pub struct Allowance { pub allowance: candid::Nat, pub expires_at: Option<u64> }

#[derive(CandidType, Deserialize)]
pub struct ApproveArgs {
  pub fee: Option<candid::Nat>,
  pub memo: Option<serde_bytes::ByteBuf>,
  pub from_subaccount: Option<serde_bytes::ByteBuf>,
  pub created_at_time: Option<u64>,
  pub amount: candid::Nat,
  pub expected_allowance: Option<candid::Nat>,
  pub expires_at: Option<u64>,
  pub spender: Account,
}

#[derive(CandidType, Deserialize)]
pub enum ApproveError {
  GenericError{ message: String, error_code: candid::Nat },
  TemporarilyUnavailable,
  Duplicate{ duplicate_of: candid::Nat },
  BadFee{ expected_fee: candid::Nat },
  AllowanceChanged{ current_allowance: candid::Nat },
  CreatedInFuture{ ledger_time: u64 },
  TooOld,
  Expired{ ledger_time: u64 },
  InsufficientFunds{ balance: candid::Nat },
}

#[derive(CandidType, Deserialize)]
pub enum Result2 { Ok(candid::Nat), Err(ApproveError) }

#[derive(CandidType, Deserialize)]
pub struct TransferFromArgs {
  pub to: Account,
  pub fee: Option<candid::Nat>,
  pub spender_subaccount: Option<serde_bytes::ByteBuf>,
  pub from: Account,
  pub memo: Option<serde_bytes::ByteBuf>,
  pub created_at_time: Option<u64>,
  pub amount: candid::Nat,
}

#[derive(CandidType, Deserialize)]
pub enum TransferFromError {
  GenericError{ message: String, error_code: candid::Nat },
  TemporarilyUnavailable,
  InsufficientAllowance{ allowance: candid::Nat },
  BadBurn{ min_burn_amount: candid::Nat },
  Duplicate{ duplicate_of: candid::Nat },
  BadFee{ expected_fee: candid::Nat },
  CreatedInFuture{ ledger_time: u64 },
  TooOld,
  InsufficientFunds{ balance: candid::Nat },
}

#[derive(CandidType, Deserialize)]
pub enum Result3 { Ok(candid::Nat), Err(TransferFromError) }

#[derive(CandidType, Deserialize)]
pub struct Name { pub name: String }

#[derive(CandidType, Deserialize)]
pub struct GetBlocksArgs { pub start: u64, pub length: u64 }

#[derive(CandidType, Deserialize)]
pub struct TimeStamp { pub timestamp_nanos: u64 }

#[derive(CandidType, Deserialize)]
pub enum CandidOperation {
  Approve{
    fee: Tokens,
    from: serde_bytes::ByteBuf,
    allowance_e8s: candid::Int,
    allowance: Tokens,
    expected_allowance: Option<Tokens>,
    expires_at: Option<TimeStamp>,
    spender: serde_bytes::ByteBuf,
  },
  Burn{
    from: serde_bytes::ByteBuf,
    amount: Tokens,
    spender: Option<serde_bytes::ByteBuf>,
  },
  Mint{ to: serde_bytes::ByteBuf, amount: Tokens },
  Transfer{
    to: serde_bytes::ByteBuf,
    fee: Tokens,
    from: serde_bytes::ByteBuf,
    amount: Tokens,
    spender: Option<serde_bytes::ByteBuf>,
  },
}

#[derive(CandidType, Deserialize)]
pub struct CandidTransaction {
  pub memo: u64,
  pub icrc1_memo: Option<serde_bytes::ByteBuf>,
  pub operation: Option<CandidOperation>,
  pub created_at_time: TimeStamp,
}

#[derive(CandidType, Deserialize)]
pub struct CandidBlock {
  pub transaction: CandidTransaction,
  pub timestamp: TimeStamp,
  pub parent_hash: Option<serde_bytes::ByteBuf>,
}

#[derive(CandidType, Deserialize)]
pub struct BlockRange { pub blocks: Vec<CandidBlock> }

#[derive(CandidType, Deserialize)]
pub enum GetBlocksError {
  BadFirstBlockIndex{ requested_index: u64, first_valid_index: u64 },
  Other{ error_message: String, error_code: u64 },
}

#[derive(CandidType, Deserialize)]
pub enum Result4 { Ok(BlockRange), Err(GetBlocksError) }

candid::define_function!(pub ArchivedBlocksRangeCallback : (GetBlocksArgs) -> (
    Result4,
  ) query);
#[derive(CandidType, Deserialize)]
pub struct ArchivedBlocksRange {
  pub callback: ArchivedBlocksRangeCallback,
  pub start: u64,
  pub length: u64,
}

#[derive(CandidType, Deserialize)]
pub struct QueryBlocksResponse {
  pub certificate: Option<serde_bytes::ByteBuf>,
  pub blocks: Vec<CandidBlock>,
  pub chain_length: u64,
  pub first_block_index: u64,
  pub archived_blocks: Vec<ArchivedBlocksRange>,
}

#[derive(CandidType, Deserialize)]
pub enum Result5 { Ok(Vec<serde_bytes::ByteBuf>), Err(GetBlocksError) }

candid::define_function!(pub ArchivedEncodedBlocksRangeCallback : (
    GetBlocksArgs,
  ) -> (Result5) query);
#[derive(CandidType, Deserialize)]
pub struct ArchivedEncodedBlocksRange {
  pub callback: ArchivedEncodedBlocksRangeCallback,
  pub start: u64,
  pub length: u64,
}

#[derive(CandidType, Deserialize)]
pub struct QueryEncodedBlocksResponse {
  pub certificate: Option<serde_bytes::ByteBuf>,
  pub blocks: Vec<serde_bytes::ByteBuf>,
  pub chain_length: u64,
  pub first_block_index: u64,
  pub archived_blocks: Vec<ArchivedEncodedBlocksRange>,
}

#[derive(CandidType, Deserialize)]
pub struct SendArgs {
  pub to: String,
  pub fee: Tokens,
  pub memo: u64,
  pub from_subaccount: Option<serde_bytes::ByteBuf>,
  pub created_at_time: Option<TimeStamp>,
  pub amount: Tokens,
}

#[derive(CandidType, Deserialize)]
pub struct Symbol { pub symbol: String }

#[derive(CandidType, Deserialize)]
pub struct TransferArgs {
  pub to: serde_bytes::ByteBuf,
  pub fee: Tokens,
  pub memo: u64,
  pub from_subaccount: Option<serde_bytes::ByteBuf>,
  pub created_at_time: Option<TimeStamp>,
  pub amount: Tokens,
}

#[derive(CandidType, Deserialize)]
pub enum TransferError1 {
  TxTooOld{ allowed_window_nanos: u64 },
  BadFee{ expected_fee: Tokens },
  TxDuplicate{ duplicate_of: u64 },
  TxCreatedInFuture,
  InsufficientFunds{ balance: Tokens },
}

#[derive(CandidType, Deserialize)]
pub enum Result6 { Ok(u64), Err(TransferError1) }

#[derive(CandidType, Deserialize)]
pub struct TransferFeeArg {}

#[derive(CandidType, Deserialize)]
pub struct TransferFee { pub transfer_fee: Tokens }

// pub struct Service(pub Principal);
// impl Service {
//   pub async fn account_balance(&self, arg0: BinaryAccountBalanceArgs) -> Result<
//     (Tokens,)
//   > { ic_cdk::call(self.0, "account_balance", (arg0,)).await }
//   pub async fn account_balance_dfx(&self, arg0: AccountBalanceArgs) -> Result<
//     (Tokens,)
//   > { ic_cdk::call(self.0, "account_balance_dfx", (arg0,)).await }
//   pub async fn account_identifier(&self, arg0: Account) -> Result<
//     (serde_bytes::ByteBuf,)
//   > { ic_cdk::call(self.0, "account_identifier", (arg0,)).await }
//   pub async fn archives(&self) -> Result<(Archives,)> {
//     ic_cdk::call(self.0, "archives", ()).await
//   }
//   pub async fn decimals(&self) -> Result<(Decimals,)> {
//     ic_cdk::call(self.0, "decimals", ()).await
//   }
//   pub async fn icrc_10_supported_standards(&self) -> Result<
//     (Vec<StandardRecord>,)
//   > { ic_cdk::call(self.0, "icrc10_supported_standards", ()).await }
//   pub async fn icrc_1_balance_of(&self, arg0: Account) -> Result<
//     (candid::Nat,)
//   > { ic_cdk::call(self.0, "icrc1_balance_of", (arg0,)).await }
//   pub async fn icrc_1_decimals(&self) -> Result<(u8,)> {
//     ic_cdk::call(self.0, "icrc1_decimals", ()).await
//   }
//   pub async fn icrc_1_fee(&self) -> Result<(candid::Nat,)> {
//     ic_cdk::call(self.0, "icrc1_fee", ()).await
//   }
//   pub async fn icrc_1_metadata(&self) -> Result<
//     (Vec<(String,MetadataValue,)>,)
//   > { ic_cdk::call(self.0, "icrc1_metadata", ()).await }
//   pub async fn icrc_1_minting_account(&self) -> Result<(Option<Account>,)> {
//     ic_cdk::call(self.0, "icrc1_minting_account", ()).await
//   }
//   pub async fn icrc_1_name(&self) -> Result<(String,)> {
//     ic_cdk::call(self.0, "icrc1_name", ()).await
//   }
//   pub async fn icrc_1_supported_standards(&self) -> Result<
//     (Vec<StandardRecord>,)
//   > { ic_cdk::call(self.0, "icrc1_supported_standards", ()).await }
//   pub async fn icrc_1_symbol(&self) -> Result<(String,)> {
//     ic_cdk::call(self.0, "icrc1_symbol", ()).await
//   }
//   pub async fn icrc_1_total_supply(&self) -> Result<(candid::Nat,)> {
//     ic_cdk::call(self.0, "icrc1_total_supply", ()).await
//   }
//   pub async fn icrc_1_transfer(&self, arg0: TransferArg) -> Result<(Result_,)> {
//     ic_cdk::call(self.0, "icrc1_transfer", (arg0,)).await
//   }
//   pub async fn icrc_21_canister_call_consent_message(
//     &self,
//     arg0: ConsentMessageRequest,
//   ) -> Result<(Result1,)> {
//     ic_cdk::call(self.0, "icrc21_canister_call_consent_message", (arg0,)).await
//   }
//   pub async fn icrc_2_allowance(&self, arg0: AllowanceArgs) -> Result<
//     (Allowance,)
//   > { ic_cdk::call(self.0, "icrc2_allowance", (arg0,)).await }
//   pub async fn icrc_2_approve(&self, arg0: ApproveArgs) -> Result<(Result2,)> {
//     ic_cdk::call(self.0, "icrc2_approve", (arg0,)).await
//   }
//   pub async fn icrc_2_transfer_from(&self, arg0: TransferFromArgs) -> Result<
//     (Result3,)
//   > { ic_cdk::call(self.0, "icrc2_transfer_from", (arg0,)).await }
//   pub async fn name(&self) -> Result<(Name,)> {
//     ic_cdk::call(self.0, "name", ()).await
//   }
//   pub async fn query_blocks(&self, arg0: GetBlocksArgs) -> Result<
//     (QueryBlocksResponse,)
//   > { ic_cdk::call(self.0, "query_blocks", (arg0,)).await }
//   pub async fn query_encoded_blocks(&self, arg0: GetBlocksArgs) -> Result<
//     (QueryEncodedBlocksResponse,)
//   > { ic_cdk::call(self.0, "query_encoded_blocks", (arg0,)).await }
//   pub async fn send_dfx(&self, arg0: SendArgs) -> Result<(u64,)> {
//     ic_cdk::call(self.0, "send_dfx", (arg0,)).await
//   }
//   pub async fn symbol(&self) -> Result<(Symbol,)> {
//     ic_cdk::call(self.0, "symbol", ()).await
//   }
//   pub async fn transfer(&self, arg0: TransferArgs) -> Result<(Result6,)> {
//     ic_cdk::call(self.0, "transfer", (arg0,)).await
//   }
//   pub async fn transfer_fee(&self, arg0: TransferFeeArg) -> Result<
//     (TransferFee,)
//   > { ic_cdk::call(self.0, "transfer_fee", (arg0,)).await }
// }

// icrc1_balance_of arg
#[derive(CandidType, Deserialize)]
pub struct Icrc1BalanceOfArg {
  pub owner: Principal,
  pub  subaccount: Option<serde_bytes::ByteBuf>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Transaction {
    pub to: Recipient,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub from_subaccount: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Recipient {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferResult {
    Ok(Nat),
    Err(CustomTransferError),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum CustomTransferError {
    GenericError { message: String, error_code: Nat },
    TemporarilyUnavailable,
    BadBurn { min_burn_amount: Nat },
    Duplicate { duplicate_of: Nat },
    BadFee { expected_fee: Nat },
    CreatedInFuture { ledger_time: u64 },
    TooOld,
    InsufficientFunds { balance: Nat },
}
