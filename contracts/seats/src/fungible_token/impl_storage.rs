use crate::{fungible_token::events, types, Seats};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::{env, json_types::U128, log, near_bindgen, require, AccountId, Balance, Promise};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

#[near_bindgen]
impl StorageManagement for Seats {
    /// Receives an attached deposit of NEAR for a given account.
    /// This is how an account is registered into the contract so it's able
    /// to own tokens.
    ///
    /// ### Parameters
    ///
    /// - `account_id`: The account_id that is receiving the deposit, that is
    /// being registered. If `null`, then the account in question is assumed to
    /// be the caller himself.
    /// - `registration_only`: This value is always assumed to be `true` by
    /// this contract, in which case any extra paid deposit is returned back to
    /// the caller.
    ///
    /// ### Return
    ///
    /// Returns the [`StorageBalance`] showing updated balances.
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let _registration_only = registration_only;

        self.assert_non_migration();
        self.assert_non_minting();

        let amount: Balance = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        if self.accounts.contains_key(&account_id) {
            log!("The account is already registered, refunding the deposit");
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            let min_balance = self.storage_balance_bounds().min.0;
            require!(
                amount >= min_balance,
                "The attached deposit is less than the minimum storage balance"
            );

            self.internal_register_account(&account_id);
            let refund = amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }
        self.internal_storage_balance_of(&account_id).unwrap()
    }

    /// Withdraw specified amount of `available` NEAR for predecessor account.
    /// As this contract always leave zero as `available` amounts for accounts,
    ///  this method will either panic or show the account_id balance
    /// information.
    ///
    /// ### Parameters
    ///
    /// - `amount`: Represents the amount of yoctoNEAR tokens to be withdrew
    /// from the user's `available` balance. If `null`, then the full
    /// `available` balance is withdrew.
    ///
    /// While storage_withdraw normally allows the caller to retrieve `
    /// available` balance, the basic Fungible Token implementation sets
    /// storage_balance_bounds.min == storage_balance_bounds.max,
    /// which means available balance will always be 0.
    ///
    /// So this implementation:
    /// - panics if `amount > 0`.
    /// - never transfers Ⓝ to caller.
    /// - returns a `storage_balance` struct if `amount` is 0.
    ///
    /// ### Return
    ///
    /// Returns the [`StorageBalance`] showing updated balances.
    #[payable]
    fn storage_withdraw(
        &mut self,
        amount: Option<U128>,
    ) -> StorageBalance {
        self.assert_non_migration();
        self.assert_non_minting();

        near_sdk::assert_one_yocto();
        let predecessor_account_id = env::predecessor_account_id();
        if let Some(storage_balance) = self.internal_storage_balance_of(&predecessor_account_id) {
            match amount {
                Some(amount) if amount.0 > 0 => {
                    env::panic_str("The amount is greater than the available storage balance");
                }
                _ => storage_balance,
            }
        } else {
            env::panic_str(
                format!("The account {} is not registered", &predecessor_account_id).as_str(),
            );
        }
    }

    /// Unregisters the predecessor account.
    ///
    /// ### Parameters
    ///
    /// - `force`: Whether the removal should be forced.
    /// If `true`, then assets from the user (token amount) are removed or
    /// burned. Otherwise if `null` then `false` is assumed, and in the `false`
    /// case then the function will fail if the user being unregistered still
    /// own assets.
    ///
    /// ###### Notes
    ///
    /// - If the user being removed is still registered as a vesting user,
    /// then that user won't get any token rewards during vesting user minting
    /// operations - ie. that user is skipped during rewards, and the tokens it
    /// would gain are effectively burned.
    ///
    /// ### Return
    ///
    /// Returns `true` iff the account was successfully unregistered;
    /// returns `false` iff the account was not registered before.
    #[payable]
    fn storage_unregister(
        &mut self,
        force: Option<bool>,
    ) -> bool {
        self.assert_non_migration();
        self.assert_non_minting();

        #[allow(unused_variables)]
        if let Some((account_id, user)) = self.internal_storage_unregister(force) {
            self.on_account_closed(account_id, user.balance.0);
            true
        } else {
            false
        }
    }

    /// Get the contract's setting for minimum and maximum of storage balance
    /// requirements for the users.
    ///
    /// ### Parameters
    ///
    /// Has no parameters.
    ///
    /// ### Return
    ///
    /// Returns the contract's settings for the [`StorageBalanceBounds`].
    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance = crate::storage_costs::user::BALANCE_REQUIREMENT;
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()),
        }
    }

    /// Gets the storage balance information of a given account_id.
    ///
    /// ### Parameters
    ///
    /// - `account_id`: Account_id that the balance information is being
    /// queried from.
    ///
    /// ### Return
    ///
    /// Optionally returns the [`StorageBalance`] information of the given
    /// user. Returns `null` if the account is not registered.
    fn storage_balance_of(
        &self,
        account_id: AccountId,
    ) -> Option<StorageBalance> {
        self.internal_storage_balance_of(&account_id)
    }
}

impl Seats {
    /// Internal method that returns the Account ID and the balance in case the account was
    /// unregistered.
    pub fn internal_storage_unregister(
        &mut self,
        force: Option<bool>,
    ) -> Option<(AccountId, types::User)> {
        near_sdk::assert_one_yocto();
        let account_id = env::predecessor_account_id();
        self.internal_storage_unregister_user(account_id, force)
    }

    /// Internal method that returns the Account ID and the balance in case the account was
    /// unregistered.
    pub fn internal_storage_unregister_user(
        &mut self,
        account_id: AccountId,
        force: Option<bool>,
    ) -> Option<(AccountId, types::User)> {
        let force = force.unwrap_or(false);

        match self.accounts.get(&account_id) {
            None => {
                log!("The account {} is not registered", &account_id);
                None
            }
            Some(user) => {
                require!(
                    (user.balance.0 == 0 && user.claim_balance.0 == 0) || force,
                    "Can't unregister the account with the positive token (or claimable token) balance without force"
                );

                self.accounts.remove(&account_id);
                self.total_supply -= user.balance.0;
                self.total_supply -= user.claim_balance.0;

                if user.balance.0 > 0 {
                    events::FtBurn {
                        owner_id: &account_id,
                        amount: &user.balance,
                        memo: Some("unregister"),
                    }
                    .emit();
                }

                Promise::new(account_id.clone()).transfer(self.storage_balance_bounds().min.0 + 1);
                Some((account_id, user))
            }
        }
    }

    fn internal_storage_balance_of(
        &self,
        account_id: &AccountId,
    ) -> Option<StorageBalance> {
        if self.accounts.contains_key(account_id) {
            Some(StorageBalance {
                total: self.storage_balance_bounds().min,
                available: 0.into(),
            })
        } else {
            None
        }
    }
}
