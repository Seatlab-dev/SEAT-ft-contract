use crate::Seats;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_sdk::{json_types::U128, near_bindgen, AccountId};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

#[near_bindgen]
impl FungibleTokenResolver for Seats {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            self.on_tokens_burned(sender_id, burned_amount)
        }
        used_amount.into()
    }
}
