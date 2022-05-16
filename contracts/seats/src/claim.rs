use crate::{fungible_token::events, Seats};
use near_sdk::{env, json_types::U128, near_bindgen};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

#[near_bindgen]
impl Seats {
    pub fn claim(&mut self) -> U128 {
        let predecessor = env::predecessor_account_id();
        let mut user = self.internal_unwrap_user(&predecessor);
        let amount = user.claim_balance;
        user.claim_balance.0 = 0;
        user.balance.0 += amount.0;
        self.accounts.insert(&predecessor, &user);

        if amount.0 > 0 {
            events::FtMint {
                owner_id: &predecessor,
                amount: &(amount.0 as u128).into(),
                memo: Some("claim"),
            }
            .emit();
        }

        amount
    }
}
