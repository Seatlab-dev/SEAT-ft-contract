use near_sdk::Gas;

pub mod contract_ext;
pub mod execution_ext;

pub use contract_ext::{ContractExt, WithAccount};
pub use execution_ext::{pretty_near, ExecutionExt};

pub trait IntoGas {
    fn into_gas(self) -> Gas;
}

impl IntoGas for u128 {
    fn into_gas(self) -> Gas {
        assert!(self < u64::MAX as u128);
        Gas(self as u64)
    }
}
