use crate::Seats;
use common::version::{version_from_env, IVersion, Version};
use near_sdk::near_bindgen;

#[cfg(not(target_arch = "wasm32"))]
use crate::SeatsContract;

#[near_bindgen]
impl IVersion for Seats {
    fn version(&self) -> Version {
        version_from_env!()
    }
}
