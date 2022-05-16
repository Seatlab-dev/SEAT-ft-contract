pub mod collections;
pub mod owners;

#[cfg(feature = "sim")]
pub mod sim;

pub use contract_version as version;

/// Stringifies some yoctoNEAR amount into it's NEAR representation.
/// The NEAR unit is not appended into the string.
///
/// Examples:
/// - 1000000000000000000000000 (yN) => "1"
/// - 1100000000000000000000000 (yN) => "1.1"
/// - 1010000000000000000000000 (yN) => "1.01"
/// - 1000000000000000000000001 (yN) => "1.000000000000000000000001"
/// - 0100000000000000000000000 (yN) => "0.1"
pub fn yocto_to_near(yoctos: u128) -> String {
    const KILO: u128 = 1000;
    const MEGA: u128 = KILO * KILO;
    const TERA: u128 = MEGA * MEGA;
    const YOTTA: u128 = TERA * TERA;

    let a = (yoctos / YOTTA).to_string();
    let b = (yoctos % YOTTA).to_string();

    let padding = 24 - b.chars().count();

    format!("{}.{}{}", a, "0".repeat(padding), b)
}

/// Currently not used, but can be useful in case a functio increases (or decreases)
/// the storage requirements and must pay (or receive) for it.
pub fn refund_deposit<R>(mut f: impl FnMut() -> R) -> R {
    use near_sdk::env;

    let initial_storage_usage = env::storage_usage();
    let storage_byte_cost = env::storage_byte_cost();
    let attached_deposit = env::attached_deposit();

    let r = f();

    let final_storage_usage = env::storage_usage();

    if final_storage_usage >= initial_storage_usage {
        // increased storage usage
        let increased_storage = final_storage_usage - initial_storage_usage;
        let increased_cost = storage_byte_cost * near_sdk::Balance::from(increased_storage);

        near_sdk::require!(
            attached_deposit >= increased_cost,
            format!("Must attach {} yoctoNEAR to cover storage", increased_cost)
        );

        let refund = attached_deposit - increased_cost;
        if refund > 1 {
            near_sdk::Promise::new(env::predecessor_account_id()).transfer(refund);
        }
    } else {
        // decreased storage usage
        let decreased_storage = initial_storage_usage - final_storage_usage;
        let decreased_cost = storage_byte_cost * near_sdk::Balance::from(decreased_storage);

        let refund = attached_deposit + decreased_cost;
        if refund > 1 {
            near_sdk::Promise::new(env::predecessor_account_id()).transfer(refund);
        }
    };

    r
}
