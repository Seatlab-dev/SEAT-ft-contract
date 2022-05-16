use crate::Seats;
use near_sdk::near_bindgen;

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

#[near_bindgen]
impl Seats {
    pub fn force_start_migration(&mut self) {
        self.assert_owner();
        self.migration_locked = true;
    }

    pub fn force_end_migration(&mut self) {
        self.assert_owner();
        self.migration_locked = false;
    }
}

impl Seats {
    pub fn assert_non_migration(&self) {
        assert!(!self.migration_locked);
    }
}
