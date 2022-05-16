#![allow(clippy::too_many_arguments)]

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LazyOption, LookupMap, UnorderedSet},
    log, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault,
};

pub mod claim;
pub mod constant_init;
pub mod event;
pub mod fungible_token;
pub mod init;
pub mod metadata;
pub mod migration;
pub mod owners;
pub mod storage_costs;
pub mod types;
pub mod version;
pub mod vesting;

pub const DECIMAL_PLACE_PADDING: u128 = u128::pow(10, metadata::DECIMALS as u32);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Seats {
    pub owners: UnorderedSet<AccountId>,

    /// Tracks how many $SEAT tokens each user has.
    ///
    /// [`AccountId`] -> [`types::User`].
    pub accounts: LookupMap<AccountId, types::User>,

    /// Total supply of all of the $SEAT tokens.
    pub total_supply: Balance,

    /// Metadata for this fungible token contract.
    pub metadata: LazyOption<FungibleTokenMetadata>,

    /// The timestmap after which the `start_mint` function will be enabled.
    pub start_timestamp: types::Timestamp,

    /// Information related to the minting operation.
    pub mint: types::MintInfo,

    /// Whether the contract is locked, for migration purposes.
    pub migration_locked: bool,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Owners,
    Accounts,
    VestingAccounts,
    VestingAccounts2 { set_name: types::SetName },
    Metadata,
}

impl Seats {
    pub fn assert_non_minting(&self) {
        assert!(!self.is_on_minting());
    }

    pub fn is_on_minting(&self) -> bool {
        !matches!(self.mint.state, types::MintState::Standby)
    }

    fn on_account_closed(
        &mut self,
        account_id: AccountId,
        balance: Balance,
    ) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(
        &mut self,
        account_id: AccountId,
        amount: Balance,
    ) {
        log!("Account @{} burned {}", account_id, amount);
    }
}
