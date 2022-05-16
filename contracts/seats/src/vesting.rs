#![allow(clippy::let_and_return)]

use crate::{
    types::{self, RewardAmount, RewardPercentage},
    Seats,
};
use near_sdk::{collections::UnorderedMap, env, near_bindgen, require, AccountId};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

#[near_bindgen]
impl Seats {
    /// Gets the total of enabled rewards from all user sets for the current block
    /// timestamp.
    pub fn get_vesting_total(&self) -> RewardAmount {
        let now = types::Timestamp::from(near_sdk::env::block_timestamp());
        let rewards = self
            .mint
            .vesting
            .sets
            .values()
            .filter_map(|set| {
                if now >= set.info.start_date && now < set.info.expiration_date {
                    Some(set.info.reward.0)
                } else {
                    None
                }
            })
            .sum::<u128>();
        RewardAmount(rewards)
    }

    #[payable]
    pub fn add_vesting_set(
        &mut self,
        name: types::SetName,
        start_delay_seconds: u32,
        expiration_delay_seconds: u32,
        reward: RewardAmount,
        users_must_claim: bool,
    ) {
        use types::Timestamp;

        require!(
            env::attached_deposit() == crate::storage_costs::USER_SET,
            &format!(
                "insufficient attached payment, required {} yoctoNEAR",
                crate::storage_costs::USER_SET
            )
        );

        self.assert_owner();
        self.assert_non_migration();
        self.assert_non_minting();

        let start_date = self.start_timestamp + Timestamp::from_seconds(start_delay_seconds);
        let expiration_date = start_date + Timestamp::from_seconds(expiration_delay_seconds);

        let user_set = self.internal_prepare_vesting_set(
            &name,
            start_date,
            expiration_date,
            reward,
            users_must_claim,
        );

        let previous = self.mint.vesting.sets.insert(&name, &user_set);

        require!(
            previous.is_none(),
            &format!("set {} is already registered", &name.0)
        );
    }

    /// Changes a registered user set.
    ///
    /// Returns the previous user set information.
    pub fn change_vesting_set(
        &mut self,
        name: types::SetName,
        new_start_date: types::Timestamp,
        new_expiration_date: types::Timestamp,
        new_reward: RewardAmount,
        new_users_must_claim: bool,
    ) -> types::VestingUserSetInfo {
        self.assert_owner();
        self.assert_non_migration();
        self.assert_non_minting();

        let mut user_set = self
            .mint
            .vesting
            .sets
            .get(&name)
            .unwrap_or_else(|| env::panic_str(&format!("missing set {}", name.0)));

        let previous_set = user_set.info.clone();
        user_set.info.reward = new_reward;
        user_set.info.start_date = new_start_date;
        user_set.info.expiration_date = new_expiration_date;
        user_set.info.users_must_claim = new_users_must_claim;

        self.mint.vesting.sets.insert(&name, &user_set);

        previous_set
    }

    /// Unregisters a user set.
    ///
    /// Receives the registration deposit back to the caller.
    ///
    /// Returns the removed user set information.
    pub fn remove_vesting_set(
        &mut self,
        name: types::SetName,
        force: Option<bool>,
    ) -> types::VestingUserSetInfo {
        self.assert_owner();
        self.assert_non_migration();
        self.assert_non_minting();

        let mut user_set = self
            .mint
            .vesting
            .sets
            .get(&name)
            .unwrap_or_else(|| env::panic_str(&format!("missing set {}", name.0)));

        match (force, user_set.accounts.is_empty()) {
            (_force @ Some(true), _empty @ false) => {
                user_set.accounts.clear();
                self.mint.vesting.sets.remove(&name);
            }
            (_force, _empty @ false) => env::panic_str(&format!("set {} is not empty", name.0)),
            (_force, _empty @ true) => {
                self.mint.vesting.sets.remove(&name);
            }
        };

        // returns the deposit used for creating a new user set
        near_sdk::Promise::new(env::predecessor_account_id())
            .transfer(crate::storage_costs::USER_SET);

        user_set.info
    }

    /// Get a vesting set information.
    pub fn get_vesting_set(
        &self,
        name: String,
    ) -> Option<types::VestingUserSetInfo> {
        self.mint
            .vesting
            .sets
            .get(&types::SetName(name))
            .map(|set| set.info)
    }

    /// Get user set names.
    pub fn get_vesting_sets(
        &self,
        from_index: Option<near_sdk::json_types::U64>,
        limit: Option<u16>,
    ) -> Vec<types::SetName> {
        let from_index = from_index.unwrap_or_else(|| 0.into()).0 as usize;
        let limit = limit.unwrap_or(u16::MAX) as usize;

        self.mint
            .vesting
            .sets
            .keys()
            .skip(from_index)
            .take(limit)
            .collect()
    }

    /// Get the members from a user set.
    pub fn get_vesting_set_users(
        &self,
        name: types::SetName,
        from_index: Option<near_sdk::json_types::U64>,
        limit: Option<u16>,
    ) -> Vec<(AccountId, RewardPercentage)> {
        let from_index = from_index.unwrap_or_else(|| 0.into()).0 as usize;
        let limit = limit.unwrap_or(u16::MAX) as usize;

        let user_set = self
            .mint
            .vesting
            .sets
            .get(&name)
            .unwrap_or_else(|| env::panic_str(&format!("missing set {}", name.0)));

        let accs: Vec<(_, _)> = user_set
            .accounts
            .iter()
            .skip(from_index)
            .take(limit)
            .collect();
        accs
    }

    #[payable]
    pub fn add_vesting_user(
        &mut self,
        set: types::SetName,
        account_id: AccountId,
        percentage: types::RewardPercentage,
    ) {
        require!(
            env::attached_deposit() == crate::storage_costs::user::VESTING,
            &format!(
                "insufficient attached payment, required {} yoctoNEAR",
                crate::storage_costs::user::VESTING
            )
        );

        self.assert_owner();
        self.assert_non_migration();
        self.assert_non_minting();

        let mut user_set = self
            .mint
            .vesting
            .sets
            .get(&set)
            .unwrap_or_else(|| env::panic_str(&format!("missing set {}", set.0)));

        self.internal_add_vesting_user(&set, &mut user_set, account_id, percentage);

        self.mint.vesting.sets.insert(&set, &user_set);
    }

    pub fn remove_vesting_user(
        &mut self,
        set: types::SetName,
        account_id: AccountId,
    ) -> RewardPercentage {
        self.assert_owner();
        self.assert_non_migration();
        self.assert_non_minting();

        let mut user_set = self
            .mint
            .vesting
            .sets
            .get(&set)
            .unwrap_or_else(|| env::panic_str(&format!("missing set {}", set.0)));

        let reward = user_set.accounts.remove(&account_id).unwrap_or_else(|| {
            env::panic_str(&format!(
                "account {} not registered on set {}",
                account_id, set.0
            ))
        });

        user_set.info.total_user_percentages.0 -= reward.0;

        self.mint.vesting.sets.insert(&set, &user_set);

        // returns the deposit used for creating a new user set
        near_sdk::Promise::new(env::predecessor_account_id())
            .transfer(crate::storage_costs::user::VESTING);

        reward
    }

    pub fn change_vesting_user(
        &mut self,
        set: types::SetName,
        account_id: AccountId,
        new_percentage: RewardPercentage,
    ) -> RewardPercentage {
        self.assert_owner();
        self.assert_non_migration();
        self.assert_non_minting();

        // checks if the percentage is not above 100%.
        new_percentage.check();

        let mut user_set = self
            .mint
            .vesting
            .sets
            .get(&set)
            .unwrap_or_else(|| env::panic_str(&format!("missing set {}", set.0)));

        let previous = user_set
            .accounts
            .insert(&account_id, &new_percentage)
            .unwrap_or_else(|| {
                env::panic_str(&format!(
                    "account {} not registered on set {}",
                    account_id, set.0
                ))
            });

        user_set.info.total_user_percentages.0 -= previous.0;
        user_set.info.total_user_percentages.0 += new_percentage.0;
        user_set.info.total_user_percentages.check();

        self.mint.vesting.sets.insert(&set, &user_set);
        previous
    }

    /// Get a user's reward from a set.
    pub fn get_vesting_user(
        &self,
        set: types::SetName,
        account_id: AccountId,
    ) -> RewardPercentage {
        let user_set = self
            .mint
            .vesting
            .sets
            .get(&set)
            .unwrap_or_else(|| env::panic_str(&format!("missing set {}", set.0)));

        user_set.accounts.get(&account_id).unwrap_or_default()
    }
}

impl Seats {
    pub fn internal_prepare_vesting_set(
        &mut self,
        name: &types::SetName,
        start_date: types::Timestamp,
        expiration_date: types::Timestamp,
        reward: RewardAmount,
        users_must_claim: bool,
    ) -> types::VestingUserSet {
        let accounts = UnorderedMap::new(crate::StorageKey::VestingAccounts2 {
            set_name: name.clone(),
        });
        let user_set = types::VestingUserSet {
            info: types::VestingUserSetInfo {
                reward,
                total_rewarded: types::RewardAmount::default(),
                generation: u32::default(),
                last_mint_timestamp: types::Timestamp::default(),
                start_date,
                expiration_date,
                total_user_percentages: types::RewardPercentage::default(),
                users_must_claim,
            },
            accounts,
        };

        user_set
    }

    pub fn internal_add_vesting_user<'user_set>(
        &mut self,
        set: &types::SetName,
        user_set: &'user_set mut types::VestingUserSet,
        account_id: AccountId,
        percentage: types::RewardPercentage,
    ) -> &'user_set mut types::VestingUserSet {
        require!(
            self.accounts.contains_key(&account_id),
            &format!("account {} is not registered", &account_id)
        );

        let previous = user_set.accounts.insert(&account_id, &percentage);

        user_set.info.total_user_percentages.0 += percentage.0;
        user_set.info.total_user_percentages.check();

        require!(
            previous.is_none(),
            &format!(
                "account {} already registered for set {}",
                &account_id, set.0
            )
        );

        user_set
    }
}
