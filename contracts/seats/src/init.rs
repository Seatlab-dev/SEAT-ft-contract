#![allow(clippy::too_many_arguments)]

use crate::{metadata, types, Seats, StorageKey};
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet},
    near_bindgen, require,
    serde::{Deserialize, Serialize},
    AccountId,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SetMembers {
    /// The set name.
    pub set: types::SetName,

    /// How many raw SEAT tokens, at maximum, will be distributed to
    /// the members of the set.
    ///
    /// 1 SEAT token has 5 decimal places and it equals to
    /// 10^5 raw SEAT tokens.
    ///
    /// Eg. the value 712345 represents 7.12345 SEAT tokens.
    pub reward: types::RewardAmount,

    /// For how many seconds this set won't mint rewards.
    pub start_delay_seconds: u32,

    /// For how many seconds, after `start_delay_seconds`, this set will still
    /// mint rewards.
    pub expiration_delay_seconds: u32,

    /// Whether members must still claim their token rewards,
    /// or if they get directly rewarded of their tokens.
    pub users_must_claim: bool,

    /// List of members registered on this set.
    ///
    /// Each member requires two values, in order:
    /// - `account_id`.
    /// - "reward percentage".
    ///
    /// The "reward percentage" is a percentage-like stringfied integer,
    /// but with more integer (mantissa, characteristic) places so that the
    /// calculations have a higher precision.  
    /// 100% is represented as "100000000000", whereas
    /// 1% is represented as "001000000000".  
    /// Lower values are percentages below 1%.
    pub members: Vec<(AccountId, types::RewardPercentage)>,
}

#[near_bindgen]
impl Seats {
    /// Initializes the SEAT FT contract.
    ///
    /// ### Parameters
    ///
    /// - `owner_id`: The contract owner.
    /// - `metadata`: FT Metadata. See [NEP-148](https://nomicon.io/Standards/Tokens/FungibleToken/Metadata) for more info.
    /// - `mint_lock_duration_seconds`: After starting a minting operation, how many seconds should must the next minting operation wait for before getting started.
    /// - `start_timestamp_seconds`: Unix timestamp before which the contract should be locked.
    #[payable]
    #[init]
    pub fn new(
        owner_id: AccountId,
        metadata: FungibleTokenMetadata,
        mint_lock_duration_seconds: u32,
        start_timestamp_seconds: u32,
    ) -> Self {
        metadata::check(&metadata);
        let mut owners = UnorderedSet::new(StorageKey::Owners);
        owners.insert(&owner_id);
        let start_timestamp = types::Timestamp::from_seconds(start_timestamp_seconds);
        let mut this = Self {
            owners,
            accounts: LookupMap::new(StorageKey::Accounts),
            total_supply: 0,
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            start_timestamp,
            mint: types::MintInfo {
                generation: u32::default(),
                lock_duration_seconds: mint_lock_duration_seconds,
                last_mint_timestamp: None,
                vesting: types::Vesting {
                    sets: UnorderedMap::new(StorageKey::VestingAccounts),
                },
                state: types::MintState::Standby,
            },
            migration_locked: false,
        };
        this.internal_register_account(&owner_id);
        this
    }

    /// Initializes the SEAT FT contract with some initial vesting/user sets.
    ///
    /// ### Parameters
    ///
    /// The same as [`new()`], with additionally:
    ///
    /// - `set_members`: List for vesting sets information to be added.
    #[init]
    #[payable]
    pub fn new_with(
        owner_id: AccountId,
        metadata: FungibleTokenMetadata,
        mint_lock_duration_seconds: u32,
        start_timestamp_seconds: u32,
        // extra parameters
        set_members: Vec<SetMembers>,
    ) -> Self {
        use std::collections::HashSet;
        use types::Timestamp;

        let mut this = Self::new(
            owner_id,
            metadata,
            mint_lock_duration_seconds,
            start_timestamp_seconds,
        );

        let mut added_members = HashSet::new();

        for SetMembers {
            set,
            members,
            reward,
            start_delay_seconds,
            expiration_delay_seconds,
            users_must_claim,
        } in set_members
        {
            let start_date = Timestamp::from_seconds(start_timestamp_seconds)
                + Timestamp::from_seconds(start_delay_seconds);
            let expiration_date = start_date + Timestamp::from_seconds(expiration_delay_seconds);
            let mut user_set = this.internal_prepare_vesting_set(
                &set,
                start_date,
                expiration_date,
                reward,
                users_must_claim,
            );

            for (member, percentage) in members {
                let is_new_member = added_members.insert(member.clone());

                if is_new_member {
                    this.internal_register_account(&member);
                }

                this.internal_add_vesting_user(&set, &mut user_set, member, percentage);
            }

            let previous = this.mint.vesting.sets.insert(&set, &user_set);
            require!(
                previous.is_none(),
                &format!("set {} is already registered", &set.0)
            );
        }

        this
    }

    /// Initializes the SEAT FT contract with some pre-determined initial
    /// configuration.
    ///
    /// ### Parameters
    ///
    /// - `owner_id`: The contract owner.
    /// - `network`: "Testnet" | "Mainnet".
    #[payable]
    #[init]
    pub fn new_const(
        owner_id: AccountId,
        network: crate::constant_init::Network,
        start_timestamp_seconds: Option<u32>,
    ) -> Self {
        use crate::constant_init;

        let network = &network;

        let metadata = constant_init::metadata();
        let mint_lock_duration_seconds = constant_init::mint_lock_duration_seconds(network);
        let start_timestamp_seconds = start_timestamp_seconds
            .unwrap_or_else(|| constant_init::start_timestamp_seconds(network));
        let set_members = constant_init::set_members(network);

        Self::new_with(
            owner_id,
            metadata,
            mint_lock_duration_seconds,
            start_timestamp_seconds,
            set_members,
        )
    }
}
