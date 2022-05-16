#![allow(clippy::needless_return)]

use crate::{fungible_token::events, types, Seats};
use near_sdk::{
    env,
    json_types::{U128, U64},
    near_bindgen, AccountId,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

#[near_bindgen]
impl Seats {
    /// Change the start_timestamp in which the minting will start being available.
    ///
    /// ### Parameters
    ///
    /// - `new_start_timestamp_seconds`: The new unix timestamp in which the
    /// `start_mint` function will be enabled.
    pub fn change_start_timestamp(
        &mut self,
        new_start_timestamp_seconds: u32,
    ) {
        self.assert_owner();
        self.assert_non_migration();
        self.assert_non_minting();

        self.start_timestamp = types::Timestamp::from_seconds(new_start_timestamp_seconds);
    }

    /// Get the start_timestamp in which the minting will start being available.
    pub fn get_start_timestamp(&self) -> types::Timestamp {
        self.start_timestamp
    }

    /// Mint some amount of raw SEAT tokens to some registered user.
    ///
    /// ### Parameters
    ///
    /// - `account_id`: The account_id that will receive the tokens.
    /// - `amount`: The amount of raw SEAT tokens to be minted.  
    /// Eg. "712345" means that 7.12345 SEAT tokens will be minted.  
    /// The maximum value is `18446744073709551615`, which is
    /// around 180 trillion SEAT tokens.
    /// - `must_claim`: Whether the user must claim the minted tokens, or if he
    /// will immediatedly receive the tokens.
    pub fn force_mint(
        &mut self,
        account_id: AccountId,
        amount: U64,
        must_claim: Option<bool>,
    ) {
        self.assert_owner();
        self.assert_non_migration();
        let user = self.internal_unwrap_user(&account_id);
        let must_claim = must_claim.unwrap_or_default();
        self.internal_user_deposit(&account_id, user, amount.0 as u128, must_claim);

        if !must_claim {
            events::FtMint {
                owner_id: &account_id,
                amount: &(amount.0 as u128).into(),
                memo: Some("force"),
            }
            .emit();
        }
    }

    /// Starts the minting procedure.
    ///
    /// Maximum reward value is 18446744073709551615 (~18 mN, in yN units),
    /// which is 18 million trillion units.
    pub fn start_mint(&mut self) -> types::MintState {
        use near_sdk::require;

        self.assert_owner();
        self.assert_non_migration();

        let now = types::Timestamp::from(near_sdk::env::block_timestamp());
        require!(
            now >= self.start_timestamp,
            &format!(
                "start_mint not yet enabled, {} nanoseconds remaining",
                self.start_timestamp.0 - now.0
            )
        );

        if let Some(last_mint_timestamp) = self.mint.last_mint_timestamp {
            let required_timestamp = last_mint_timestamp
                + types::Timestamp::from_seconds(self.mint.lock_duration_seconds);

            require!(
                now >= required_timestamp,
                &format!(
                    "start_mint locked, {} nanoseconds remaining",
                    required_timestamp.0 - now.0
                )
            );
        }

        self.mint.last_mint_timestamp = Some(now);
        self.mint.state = types::MintState::Vesting {
            set_offset: 0,
            user_offset: 0,
        };
        self.mint.state.clone()
    }

    pub fn force_end_mint(&mut self) {
        self.assert_owner();
        self.assert_non_migration();
        self.internal_end_minting();
    }

    /// Progress the rewards distribution/minting operation. This should be
    /// repeatedly called untill the MintState goes back to Standby.
    ///
    /// Depending on the MintState, in order, the rewards are distributed to the
    /// Vesting users.
    ///
    /// The limit sets how many users will receive their rewards distribution.
    /// Also the limit counts for how many different sets will be analyzed.
    ///
    /// If without a limit of users, a limit of 100 users/sets is assumed, which
    /// should not hit gas limit errors.
    pub fn step_mint(
        &mut self,
        limit: Option<u8>,
    ) -> types::MintState {
        use common::collections::KeyValueAccess;
        use types::MintState;

        self.assert_non_migration();

        let limit = limit.unwrap_or(100);

        match self.mint.state {
            MintState::Standby => env::panic_str("minting not in progress"),

            // minting for vesting users
            MintState::Vesting {
                mut set_offset,
                mut user_offset,
            } => {
                let mut mint_events = vec![];

                // checks if should move to the next step
                let sets_len = self.mint.vesting.sets.len();

                if set_offset as u64 >= sets_len {
                    // moves to the next step (end)
                    return self.internal_end_minting();
                }

                let mut i = 0;
                'outer: while i < limit {
                    let (set_key, mut set_value) =
                        match self.mint.vesting.sets.index(set_offset as u64) {
                            None => break 'outer,
                            Some(e) => e,
                        };

                    // check if this user set has already finished
                    // or if it's not enabled
                    let user_len = set_value.accounts.len();
                    let last_mint_timestamp = self.mint.last_mint_timestamp.unwrap_or_default();
                    if
                    // no more members on the set
                    user_offset >= user_len
                    // user set not yet started
                    || last_mint_timestamp < set_value.info.start_date
                    // user set expired 
                    || last_mint_timestamp >= set_value.info.expiration_date
                    {
                        // for when the set was active
                        if user_offset >= user_len {
                            // last update to set information
                            set_value.info.generation += 1;
                            set_value.info.last_mint_timestamp =
                                self.mint.last_mint_timestamp.unwrap();
                            // UnorderedMap doesn't offer an &mut access to the
                            // values, so even if we already know the element index,
                            // we need to re-calculate it with the set_name (the key)
                            self.mint.vesting.sets.insert(&set_key, &set_value);
                        }

                        // moves to the next user set
                        set_offset += 1;
                        user_offset = 0;
                        i += 1;
                        continue 'outer;
                    }

                    let users_must_claim = set_value.info.users_must_claim;

                    while i < limit {
                        let (account_id, percentage) = match set_value.accounts.index(user_offset) {
                            None => {
                                // moves to the next user set
                                set_offset += 1;
                                user_offset = 0;
                                i += 1;

                                // last update to set information
                                set_value.info.generation += 1;
                                set_value.info.last_mint_timestamp =
                                    self.mint.last_mint_timestamp.unwrap();
                                // UnorderedMap doesn't offer an &mut access to the
                                // values, so even if we already know the element
                                // index, we need to re-calculate it with the
                                // set_name (the key)
                                self.mint.vesting.sets.insert(&set_key, &set_value);

                                continue 'outer;
                            }
                            Some(e) => e,
                        };

                        let reward = percentage.to_reward(set_value.info.reward);

                        // this takes self by ref mut
                        let deposited =
                            self.try_internal_deposit(&account_id, reward, users_must_claim);
                        // note: in case the deposit has failed
                        // (user unregistered, then the tokens are effectivelly
                        //  burned, ie. not created)
                        if !users_must_claim && deposited {
                            mint_events.push((account_id, U128(reward)));
                        }

                        set_value.info.total_rewarded.0 += reward;

                        // moves to the next user of this set
                        user_offset += 1;
                        i += 1;
                    }

                    // the set may still have more users, but the step_mint must stop
                    // to limit gas usage. For now, we just update the set to save
                    // the total_reward that it has increased
                    //
                    // at a later point it's generation will also get updated
                    self.mint.vesting.sets.insert(&set_key, &set_value);
                }

                // emit a batch of minting events
                let mint_events = mint_events
                    .iter()
                    .map(|(account_id, amount)| events::FtMint {
                        owner_id: account_id,
                        amount,
                        memo: None,
                    })
                    .collect::<Vec<_>>();
                if !mint_events.is_empty() {
                    events::FtMint::emit_many(&mint_events);
                }

                // checks (again) if should move to the next step
                if set_offset as u64 >= sets_len {
                    // moves to the next step (end)
                    return self.internal_end_minting();
                } else {
                    // could not advance to the next state
                    self.mint.state = MintState::Vesting {
                        set_offset,
                        user_offset,
                    };
                }
                return self.mint.state.clone();
            }
        }
    }

    pub fn get_mint_state(&self) -> types::MintState {
        self.mint.state.clone()
    }
}

impl Seats {
    pub fn internal_end_minting(&mut self) -> types::MintState {
        self.mint.generation += 1;
        self.mint.state = types::MintState::Standby;
        self.mint.state.clone()
    }
}
