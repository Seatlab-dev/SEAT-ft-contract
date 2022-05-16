use crate::{types, Seats};
use near_sdk::{
    env, json_types::U128, log, near_bindgen, require, AccountId, Balance, PromiseResult,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

pub mod events;
pub mod impl_core;
pub mod impl_resolver;
pub mod impl_storage;
pub mod mint;

#[near_bindgen]
impl Seats {
    pub fn get_user(
        &self,
        account_id: AccountId,
    ) -> types::User {
        self.internal_unwrap_user(&account_id)
    }
}

impl Seats {
    pub fn internal_unwrap_user(
        &self,
        account_id: &AccountId,
    ) -> types::User {
        self.accounts.get(account_id).unwrap_or_else(|| {
            env::panic_str(format!("The account {} is not registered", &account_id).as_str())
        })
    }

    pub fn internal_unwrap_balance_of(
        &self,
        account_id: &AccountId,
    ) -> Balance {
        self.internal_unwrap_user(account_id).balance.0
    }

    /// Tries to deposit some amount to the user.
    ///
    /// If user is not registered, returns `false`. Otherwise, returns `true`.
    #[must_use]
    pub fn try_internal_deposit(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
        must_claim: bool,
    ) -> bool {
        match self.accounts.get(account_id) {
            Some(user) => {
                self.internal_user_deposit(account_id, user, amount, must_claim);
                true
            }
            None => false,
        }
    }

    pub fn internal_user_deposit(
        &mut self,
        account_id: &AccountId,
        mut user: types::User,
        amount: Balance,
        must_claim: bool,
    ) {
        if must_claim {
            user.claim_balance = user
                .claim_balance
                .0
                .checked_add(amount)
                .unwrap_or_else(|| env::panic_str("Claim balance overflow"))
                .into();
        } else {
            user.balance = user
                .balance
                .0
                .checked_add(amount)
                .unwrap_or_else(|| env::panic_str("Balance overflow"))
                .into();
        };

        self.accounts.insert(account_id, &user);
        self.total_supply = self
            .total_supply
            .checked_add(amount)
            .unwrap_or_else(|| env::panic_str("Total supply overflow"));
    }

    pub fn internal_withdraw(
        &mut self,
        account_id: &AccountId,
        amount: Balance,
    ) {
        let user = self.internal_unwrap_user(account_id);
        self.internal_user_withdraw(account_id, user, amount)
    }

    pub fn internal_user_withdraw(
        &mut self,
        account_id: &AccountId,
        mut user: types::User,
        amount: Balance,
    ) {
        require!(
            user.balance.0 >= amount,
            "The account doesn't have enough balance"
        );
        user.balance.0 -= amount;

        self.accounts.insert(account_id, &user);

        require!(self.total_supply >= amount, "Total supply underflow");
        self.total_supply -= amount;
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        require!(
            sender_id != receiver_id,
            "Sender and receiver should be different"
        );
        require!(amount > 0, "The amount should be a positive number");

        let sender = self.internal_unwrap_user(sender_id);

        match self.accounts.get(receiver_id) {
            // normal transfer
            Some(receiver) => {
                self.internal_user_withdraw(sender_id, sender, amount);
                self.internal_user_deposit(receiver_id, receiver, amount, false);
                events::FtTransfer {
                    old_owner_id: sender_id,
                    new_owner_id: receiver_id,
                    amount: &U128(amount),
                    memo: memo.as_deref(),
                }
                .emit();
            }

            // receiver doesn't exist
            None => {
                env::panic_str(format!("The account {} is not registered", &receiver_id).as_str())
            }
        }
    }

    pub fn internal_register_account(
        &mut self,
        account_id: &AccountId,
    ) {
        let previous_account = self.accounts.insert(account_id, &types::User::default());

        require!(
            previous_account.is_none(),
            "The account is already registered"
        );
    }

    /// Internal method that returns the amount of burned tokens in a corner case
    /// when the sender has deleted (unregistered) their account while the
    /// `ft_transfer_call` was still in flight.
    /// Returns (Used token amount, Burned token amount).
    pub fn internal_ft_resolve_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> (u128, u128) {
        let amount: Balance = amount.into();

        // Get the unused amount from the `ft_on_transfer` call result.
        let unused_amount = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount, unused_amount.0)
                } else {
                    amount
                }
            }
            PromiseResult::Failed => amount,
        };

        match (unused_amount, self.accounts.get(sender_id)) {
            // The Sender successfully sent all tokens, and he no longer exists.
            (0, None) => (amount, 0),

            // The Sender successfully sent all tokens.
            (0, Some(_sender)) => (amount, 0),

            // The Receiver intends to return some tokens, but the Sender no longer
            // exists.
            //
            // So some of the Receiver's tokens may simply get burned.
            (unused_amount, None) => {
                let mut receiver = self.accounts.get(&receiver_id).unwrap_or_default();

                // Check if the Receiver exists, or if he already spent it's tokens
                // (which then maybe can't be burned)
                match receiver.balance.0 {
                    // None of it's tokens can be burned.
                    0 => {
                        (
                            // The amount that the Sender spent.
                            amount, //
                            // The amount that the Receiver burned.
                            0,
                        )
                    }
                    // Some tokens can be burned.
                    receiver_balance => {
                        // Burns at most what he has on the balance.
                        let burn_amount = std::cmp::min(receiver_balance, unused_amount);
                        receiver.balance.0 -= burn_amount;
                        self.accounts.insert(&receiver_id, &receiver);
                        log!("The account of the sender was deleted");
                        events::FtBurn {
                            owner_id: &receiver_id,
                            amount: &U128(burn_amount),
                            memo: Some("refund"),
                        }
                        .emit();
                        (
                            // The amount that the Sender spent.
                            amount, //
                            // The amount that the Receiver burned.
                            burn_amount,
                        )
                    }
                }
            }

            // The Receiver intends to return some tokens, and the Sender still
            // exists.
            (unused_amount, Some(mut sender)) => {
                let mut receiver = self.accounts.get(&receiver_id).unwrap_or_default();

                // Check if the Receiver exists, or if he already spent it's tokens
                // (which then maybe can't be returned)
                match receiver.balance.0 {
                    // None of it's tokens can be returned.
                    0 => {
                        (
                            // The amount that the Sender spent.
                            amount, //
                            // The amount that the Receiver burned.
                            0,
                        )
                    }
                    // Some tokens can be returned.
                    receiver_balance => {
                        // Returns at most what he has on the balance.
                        let return_amount = std::cmp::min(receiver_balance, unused_amount);

                        // Decrements from the Receiver.
                        receiver.balance.0 -= return_amount;
                        self.accounts.insert(&receiver_id, &receiver);

                        // Increments from the Sender.
                        sender.balance.0 += return_amount;

                        self.accounts.insert(sender_id, &sender);

                        events::FtTransfer {
                            old_owner_id: &receiver_id,
                            new_owner_id: sender_id,
                            amount: &U128(return_amount),
                            memo: Some("refund"),
                        }
                        .emit();

                        (
                            // The amount that the Sender spent.
                            amount - return_amount, //
                            // The amount that the Receiver burned.
                            0,
                        )
                    }
                }
            }
        }
    }
}
