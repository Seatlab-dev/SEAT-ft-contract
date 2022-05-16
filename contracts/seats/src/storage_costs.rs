use near_sdk::Balance;
use near_units::parse_near;

/// Cost for deploying the contract.
///
/// 2_211_460_000_000_000_000_000_000 (~2.3 N)
pub const CONTRACT: Balance = parse_near!("3 N");

pub mod user {
    use super::*;

    /// Cost for tracking a token-owning user.
    ///
    /// 1_419_330_452_312_500_000_000 (~1.5 mN)
    pub const TOKEN_OWNER: Balance = parse_near!("2 mN");

    /// Maximum cost for tracking a vesting user.
    ///
    /// 4_910_000_000_000_000_000_000 (~4.9 mN)
    pub const VESTING: Balance = parse_near!("5 mN");

    /// Cost for registering a user, wort-case scenario.
    ///
    /// ~2 mN.
    pub const BALANCE_REQUIREMENT: Balance = TOKEN_OWNER;
}

/// Cost for registering a new user set, worst-case scenario.
///
/// 5_320_000_000_000_000_000_000 (~5.3 mN)
pub const USER_SET: Balance = parse_near!("6 mN");
