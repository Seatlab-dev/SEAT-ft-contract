use crate::Seats;
use common::owners::Owners;
use near_sdk::{env, near_bindgen, require, AccountId};

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

#[near_bindgen]
impl Seats {
    pub fn assert_owner(&self) {
        let predecessor = env::predecessor_account_id();
        require!(
            self.owners.contains(&predecessor),
            &format!("The account {} is not a contract owner", predecessor)
        );
    }
}

#[near_bindgen]
impl Owners for Seats {
    /// Adds a new owner.  
    ///
    /// Returns `true` if it's a newly added owner.  
    /// Returns `false` if the owner was already added.
    ///
    /// ### Parameters
    ///
    /// - `owner_id`: the AccountId of the new owner being added.
    fn add_owner(
        &mut self,
        owner_id: AccountId,
    ) -> bool {
        self.assert_owner();
        self.owners.insert(&owner_id)
    }

    /// Removes a owner.  
    ///
    /// Returns `true` if such owner was removed.  
    /// Returns `false` if the owner wasn't added in the first place.
    ///
    /// ### Parameters
    ///
    /// - `owner_id`: the AccountId of the existing owner being removed.
    fn remove_owner(
        &mut self,
        owner_id: AccountId,
    ) -> bool {
        self.assert_owner();
        self.owners.remove(&owner_id)
    }

    /// Checks if the given account is an owner.  
    ///
    /// Returns `true` if it is, and `false` otherwise.
    ///
    /// ### Parameters
    ///
    /// - `owner_id`: The AccountId of the owner being checked.
    fn is_owner(
        &self,
        owner_id: AccountId,
    ) -> bool {
        self.owners.contains(&owner_id)
    }

    /// Get a list of the owners' account_ids.
    ///
    /// ### Parameters
    ///
    /// - `from_index`: How many owners to skip.
    /// - `limit`: How many owners to show.
    fn get_owners(
        &self,
        from_index: Option<near_sdk::json_types::U128>,
        limit: Option<u16>,
    ) -> Vec<AccountId> {
        let from_index = from_index.unwrap_or_else(|| 0.into()).0 as usize;
        let limit = limit.unwrap_or(u16::MAX) as usize;
        self.owners.iter().skip(from_index).take(limit).collect()
    }
}
