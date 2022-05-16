use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
    AccountId,
};
use serde_with::{serde_as, FromInto};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    /// The amount of SEAT tokens that the user has.
    pub balance: U128,
    /// The amount of SEAT tokens that the user can claim.
    pub claim_balance: U128,
}

impl Default for User {
    fn default() -> Self {
        Self {
            balance: 0.into(),
            claim_balance: 0.into(),
        }
    }
}

/// Specifies an amount of raw SEAT tokens.
///
/// 1 SEAT = 100_000 raw tokens.
/// 0.1 SEAT = 10_000 raw tokens.
/// 0.00001 SEAT = 1 raw token.
#[serde_as]
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq, Debug, Default,
)]
#[serde(transparent)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardAmount(
    #[serde_as(as = "FromInto<U128>")]
    //
    pub u128,
);

/// Specifies some percentage-like value of some SEAT raw tokens amount.
///
/// - 100% = 100_000_000_000.
/// - 1% = 001_000_000_000.
#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Debug,
    Default,
)]
#[serde(transparent)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardPercentage(
    #[serde_as(as = "FromInto<U64>")]
    //
    pub u64,
);

impl RewardPercentage {
    const MAX: Self = Self(100_000_000_000);

    pub fn new(percentage: u64) -> Self {
        let this = Self(percentage);
        this.check();
        this
    }

    /// Checks if the percentage is not above 100%.
    pub fn check(&self) {
        near_sdk::require!(
            *self <= Self::MAX,
            &format!("percentage {} is too high", self.0)
        );
    }

    pub fn to_reward(
        &self,
        reward: RewardAmount,
    ) -> u128 {
        (self.0 as u128 * reward.0 as u128) / (Self::MAX.0 as u128)
    }
}

impl From<u64> for RewardPercentage {
    fn from(r: u64) -> Self {
        Self::new(r)
    }
}

/// Represents a timestamp, in nanoseconds.
#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Debug,
    Default,
)]
#[serde(transparent)]
#[serde(crate = "near_sdk::serde")]
pub struct Timestamp(
    #[serde_as(as = "FromInto<U64>")]
    //
    pub u64,
);

impl From<u64> for Timestamp {
    fn from(t: u64) -> Self {
        Self(t)
    }
}

impl std::ops::Add for Timestamp {
    type Output = Self;

    fn add(
        self,
        rhs: Self,
    ) -> Self::Output {
        Self::from(self.0.add(rhs.0))
    }
}

impl std::ops::Sub for Timestamp {
    type Output = Self;

    fn sub(
        self,
        rhs: Self,
    ) -> Self::Output {
        Self::from(self.0.sub(rhs.0))
    }
}

impl Timestamp {
    const SECS_TO_NANO: u64 = 1_000_000_000;

    pub fn from_seconds(seconds: u32) -> Self {
        Self(seconds as u64 * Self::SECS_TO_NANO)
    }

    pub fn seconds_part(&self) -> u32 {
        (self.0 / Self::SECS_TO_NANO) as u32
    }

    pub fn nanoseconds_part(&self) -> u32 {
        (self.0 % Self::SECS_TO_NANO) as u32
    }
}

/// Represents a set name.
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
#[serde(transparent)]
#[serde(crate = "near_sdk::serde")]
pub struct SetName(pub String);

impl SetName {
    pub fn new(name: String) -> Self {
        near_sdk::require!(name.len() <= 64, &format!("set name {} is too big", &name));
        Self(name)
    }
}

impl From<&str> for SetName {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

/// Information about minting operations.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MintInfo {
    /// Counter of how many minting procedures has finished,
    /// and the timestmap of the last one.
    pub generation: u32,
    /// Number of seconds that the next start_mint procedure should wait
    /// after the last start_mint operation.
    pub lock_duration_seconds: u32,
    /// The last time the minting procedure has started.
    pub last_mint_timestamp: Option<Timestamp>,
    /// Information related to vesting.
    pub vesting: Vesting,
    /// The state in which the minting operation is at.
    pub state: MintState,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Vesting {
    /// Sets and their Accounts that receive for vesting.
    pub sets: UnorderedMap<SetName, VestingUserSet>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct VestingUserSet {
    pub info: VestingUserSetInfo,
    /// The members on this set.
    pub accounts: UnorderedMap<AccountId, RewardPercentage>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct VestingUserSetInfo {
    /// How much raw SEAT tokens will be distributed to this set for each minting
    /// operation.
    pub reward: RewardAmount,
    /// How much raw SEAT tokens were rewarded in total by this set.
    pub total_rewarded: RewardAmount,
    /// Counter of how many minting procedures had finished.
    pub generation: u32,
    /// The timestamp of the last time this set has minted.
    pub last_mint_timestamp: Timestamp,
    /// The date in which this set should start rewarding it's members.
    pub start_date: Timestamp,
    /// The date in which this set should stop rewarding it's members.
    pub expiration_date: Timestamp,
    /// The sum of all users percentages from this set.
    ///
    /// Must always be equal or below RewardPercentage::MAX.
    pub total_user_percentages: RewardPercentage,
    /// Whether users must still claim their token rewards,
    /// or they get directly rewarded of their tokens.
    pub users_must_claim: bool,
}

#[derive(Clone, PartialEq, Debug, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "t", content = "c")]
pub enum MintState {
    /// No minting procedure is happening.
    Standby,
    /// Minting for vesting users.
    Vesting {
        /// How many set have it's users already received their rewards.
        set_offset: u16,

        /// For a given set, how many users have been already received
        /// their rewards.
        user_offset: u64,
    },
}
