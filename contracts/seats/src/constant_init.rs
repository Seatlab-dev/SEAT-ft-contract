#![allow(clippy::identity_op)]

use crate::{
    init::SetMembers,
    types::{self, RewardAmount},
};
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

pub const SECOND: u32 = 1;
pub const MINUTE: u32 = 60 * SECOND;
pub const HOUR: u32 = 60 * MINUTE;
pub const DAY: u32 = 24 * HOUR;
pub const YEAR: u32 = 365 * DAY;
pub const MONTH: u32 = YEAR / 12;

/// Differentiates the network target.
///
/// Testnet timing durations are reduced, and in overall,
/// a day from mainnet = a minute in testnet.
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Network {
    Testnet,
    Mainnet,
}

pub fn metadata() -> FungibleTokenMetadata {
    use crate::metadata;
    use near_contract_standards::fungible_token::metadata::FT_METADATA_SPEC;

    let metadata = FungibleTokenMetadata {
        // indicates that a Fungible Token contract adheres to the current
        // versions of a Metadata and the Fungible Token Core specs.
        // This will allow consumers of the Fungible Token to know if they
        // support the features of a given contract.
        spec: FT_METADATA_SPEC.to_string(),
        name: metadata::NAME.to_string(),
        symbol: metadata::SYMBOL.to_string(),
        icon: Some(metadata::ICON.to_string()),
        reference: metadata::REFERENCE,
        reference_hash: metadata::REFERENCE_HASH,
        decimals: metadata::DECIMALS,
    };
    metadata::check(&metadata);
    metadata
}

pub fn mint_lock_duration_seconds(network: &Network) -> u32 {
    match network {
        Network::Testnet => {
            1 * MINUTE
            // gives 20s leeway, in case the timing needs to be antecipated
            - 20 * SECOND
        }
        Network::Mainnet => {
            1 * DAY
            // gives 1h leeway, in case the timing needs to be antecipated
            - 1 * HOUR
        }
    }
}

pub fn start_timestamp_seconds(network: &Network) -> u32 {
    let now = types::Timestamp::from(near_sdk::env::block_timestamp()).seconds_part();
    let delay = match network {
        Network::Testnet => 1 * MINUTE,
        Network::Mainnet => 1 * DAY,
    };

    delay + now as u32
}

#[allow(clippy::inconsistent_digit_grouping)]
pub fn set_members(network: &Network) -> Vec<SetMembers> {
    let scaled_month = match network {
        // 365 "days" / 12(months/year) = (30days + 10h average) per month
        Network::Mainnet => MONTH,
        // same as above, but changes days for minutes:
        // 365 "minutes" / 12(months/year) = (1825s average) per scaled month
        // = (30m + 25s average) per scaled month
        Network::Testnet => 365 * MINUTE / 12,
    };

    // let example: &AccountId = &"example.near".parse().unwrap();

    vec![
        SetMembers {
            set: "seed".into(),
            // 88,313.52834
            reward: RewardAmount(88_313_52834),
            // Daily mint starts immediately
            start_delay_seconds: 0,
            // ending after 18 months
            expiration_delay_seconds: 18 * scaled_month,
            users_must_claim: true,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 2,535,000
        },
        SetMembers {
            set: "presale".into(),
            // 270,863.01370
            reward: RewardAmount(270_863_01370),
            // Daily mint starts immediately
            start_delay_seconds: 0,
            // ending after 12 months
            expiration_delay_seconds: 12 * scaled_month,
            users_must_claim: true,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 10,985,000
        },
        SetMembers {
            set: "ido".into(),
            // 34,821.42857
            reward: RewardAmount(34_821_42857),
            // Daily mint starts after 1 month
            start_delay_seconds: 1 * scaled_month,
            // lasting 3 months (ends on month 4)
            expiration_delay_seconds: 3 * scaled_month,
            users_must_claim: false,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 1,056,250
        },
        SetMembers {
            set: "treasury".into(),
            // 219,931.50685
            reward: RewardAmount(219_931_50685),
            // Daily mint starts after 3 months
            start_delay_seconds: 3 * scaled_month,
            // lasting 12 months (ends on month 15)
            expiration_delay_seconds: 12 * scaled_month,
            users_must_claim: false,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 0
        },
        SetMembers {
            set: "airdrop".into(),
            // 23,150.684932
            reward: RewardAmount(23_150_68493),
            // Daily mint starts immediately
            start_delay_seconds: 0,
            // ending after 12 months
            expiration_delay_seconds: 12 * scaled_month,
            users_must_claim: false,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 0
        },
        SetMembers {
            set: "advisors-and-marketing".into(),
            // 208,356.16438
            reward: RewardAmount(208_356_16438),
            // Daily mint starts after 9 months
            start_delay_seconds: 9 * scaled_month,
            // lasting 12 months (ends on month 21)
            expiration_delay_seconds: 12 * scaled_month,
            users_must_claim: false,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 0
        },
        SetMembers {
            set: "strategic-partnerships".into(),
            // 110,855.26316
            reward: RewardAmount(110_855_26316),
            // Daily mint starts after 6 months
            start_delay_seconds: 6 * scaled_month,
            // lasting 15 months (ends on month 21)
            expiration_delay_seconds: 15 * scaled_month,
            users_must_claim: false,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 0
        },
        SetMembers {
            set: "team".into(),
            // 92,653.50877
            reward: RewardAmount(92_653_50877),
            // Daily mint starts after 12 months
            start_delay_seconds: 12 * scaled_month,
            // lasting 30 months (ends on month 42)
            expiration_delay_seconds: 30 * scaled_month,
            users_must_claim: false,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 0
        },
        SetMembers {
            set: "community-and-ecosystem".into(),
            // 162,054.79452
            reward: RewardAmount(162_054_79452),
            // Daily mint starts immediately
            start_delay_seconds: 0,
            // ending after 60 months
            expiration_delay_seconds: 60 * scaled_month,
            users_must_claim: false,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 0
        },
        SetMembers {
            set: "founders".into(),
            // 77,168.94977
            reward: RewardAmount(77_168_94977),
            // Daily mint starts after 15 months
            start_delay_seconds: 15 * scaled_month,
            // lasting 36 months (ends on month 51)
            expiration_delay_seconds: 36 * scaled_month,
            users_must_claim: false,
            members: vec![
                // member list
                // (example.clone(), RewardPercentage(100_000000000)),
            ],
            // day0 reward of 0
        },
    ]
}
