use crate::Seats;
use near_contract_standards::fungible_token::{
    core::FungibleTokenCore,
    core_impl::{ext_fungible_token_receiver, ext_self},
};
use near_sdk::{
    env, json_types::U128, near_bindgen, require, AccountId, Balance, Gas, PromiseOrValue,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);

const NO_DEPOSIT: Balance = 0;

#[near_bindgen]
impl FungibleTokenCore for Seats {
    /// Simple transfer to a receiver.
    ///
    /// ### Parameters
    ///
    /// - `receiver_id`: AccountId of the receiver.
    /// - `amount`: Stringfied 128-bit unsigned integer representing the amount
    /// of raw SEAT tokens being transferred.
    /// - `memo`: Used by use cases that may benefit from indexing or providing
    /// information for a transfer.
    ///
    ///
    #[payable]
    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
    ) {
        self.assert_non_migration();
        self.assert_non_minting();

        near_sdk::assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();

        self.internal_transfer(&sender_id, &receiver_id, amount, memo);
    }

    /// Transfer tokens and then calls the `ft_on_transfer` method on the
    /// receiver contract - which acknowledges or denies the transfer -
    /// and then makes a callback on the `ft_resolve_transfer` method back to
    /// the fungible token contract. This workflow can, for example, be used to
    /// "attach" tokens as a "deposit" in a call to a receiver contract.
    ///
    /// ### Parameters
    ///
    /// - `receiver_id`: AccountId of the receiver.
    /// - `amount`: Stringfied 128-bit unsigned integer representing the amount
    /// of raw SEAT tokens being transferred.
    /// - `memo`: Used by use cases that may benefit from indexing or providing
    /// information for a transfer.
    /// - `msg`: Sent as the `msg` parameter on the `receiver_id`'s
    /// `ft_on_transfer` method. Can send arbitrary information that the
    /// receiver may require.
    ///
    /// ###### Notes
    ///
    /// - The `amount` of `"712345"` represents `7.12345` SEAT tokens.
    /// - The `receiver_id`, as a contract, must implement the `ft_on_transfer`
    /// method, which indicates whether the transfer was accepted or not.
    /// Please check NEP-141 for more information.
    ///
    /// ### Return
    ///
    /// Returns a stringfied 128-bit unsigned integer representation of the
    /// amount of tokens that were used from the sender.
    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.assert_non_migration();
        self.assert_non_minting();

        near_sdk::assert_one_yocto();
        require!(
            env::prepaid_gas() > GAS_FOR_FT_TRANSFER_CALL + GAS_FOR_RESOLVE_TRANSFER,
            "More gas is required"
        );
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_transfer(&sender_id, &receiver_id, amount, memo);
        // Initiating receiver's call and the callback
        ext_fungible_token_receiver::ft_on_transfer(
            sender_id.clone(),
            amount.into(),
            msg,
            receiver_id.clone(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::ft_resolve_transfer(
            sender_id,
            receiver_id,
            amount.into(),
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

    /// Gets the total supply of raw SEAT tokens.
    ///
    /// ### Return
    ///
    /// Returns a stringfied 128-bit unsigned integer representation of the
    /// total amount of raw SEAT tokens that exist in the contract.
    ///
    /// Eg. `"712345"`, which represents `7.12345` SEAT tokens.
    fn ft_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    /// Get the balance of raw SEAT tokens of a user.
    ///
    /// ### Parameters
    ///
    /// - `account_id`: string - The account_id of the user being queried.
    ///
    /// ### Return
    ///
    /// Returns a stringfied 128-bit unsigned integer representation of the
    /// amount of raw SEAT tokens that the user owns.
    fn ft_balance_of(
        &self,
        account_id: AccountId,
    ) -> U128 {
        self.accounts.get(&account_id).unwrap_or_default().balance
    }
}
