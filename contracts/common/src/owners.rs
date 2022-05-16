use near_sdk::AccountId;

pub trait Owners {
    fn add_owner(
        &mut self,
        owner_id: AccountId,
    ) -> bool;

    /// Removes a owner.  
    ///
    /// Returns `true` if such owner was removed.  
    /// Returns `false` if the owner wasn't added in the first place.
    fn remove_owner(
        &mut self,
        owner_id: AccountId,
    ) -> bool;

    /// Checks if the given account is an owner.  
    ///
    /// Returns `true` if it is, and `false` otherwise.
    fn is_owner(
        &self,
        owner_id: AccountId,
    ) -> bool;

    /// Show owners.
    ///
    /// Returns a list of `AccountId`'s.
    fn get_owners(
        &self,
        from_index: Option<near_sdk::json_types::U128>,
        limit: Option<u16>,
    ) -> Vec<AccountId>;
}
