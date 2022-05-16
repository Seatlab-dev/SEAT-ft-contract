#![allow(clippy::needless_borrow)]
#![allow(clippy::zero_prefixed_literal)]

use common::sim::ContractExt;
pub use near_sdk::{
    json_types::{Base64VecU8, U128, U64},
    serde_json::json,
    AccountId,
};
use near_sdk_sim::{init_simulator, ContractAccount, UserAccount};
use near_units::parse_near;
use seats::{types::SetName, SeatsContract};
use stlb_seats_ft as seats;

pub mod utils;

const TERA: u128 = 1_000_000_000_000;

fn init(
    mint_lock_duration_seconds: u32,
    start_timestamp_seconds: u32,
    set_members: Option<Vec<seats::init::SetMembers>>,
    extra_deposit: u128,
) -> (
    UserAccount,
    ContractAccount<SeatsContract>,
    UserAccount,
    UserAccount,
    UserAccount,
) {
    use utils::{long_name, long_user};
    let root = init_simulator(None);
    let seats_id: AccountId = long_user("seats");

    let metadata = near_contract_standards::fungible_token::metadata::FungibleTokenMetadata {
        // required by validation
        spec: "ft-1.0.0".to_string(),
        name: long_name("-metadata-name", 256),
        symbol: long_name("-metadata-symbol", 256),
        icon: Some(long_name("-metadata-icon", 512)),
        reference: Some(long_name("-metadata-reference", 256)),
        // required to be 32 bytes by validation
        reference_hash: Some([0; 32].to_vec().into()),
        decimals: 0,
    };

    let seats = utils::setup_seats(
        &root,
        &seats_id.to_string(),
        mint_lock_duration_seconds,
        start_timestamp_seconds,
        Some(metadata),
        set_members,
        extra_deposit,
    );

    let alice = root.create_user(long_user("alice"), parse_near!("10 kN"));
    let bob = root.create_user(long_user("bob"), parse_near!("10 kN"));
    let carol = root.create_user(long_user("carol"), parse_near!("10 kN"));

    (root, seats, alice, bob, carol)
}

/// Registers a user that will be a token owner and a vesting user.
#[test]
fn basic_cost_test() {
    let (ref root, ref seats, ref alice, ref _bob, ref _carol) = init(1, 0, None, 0);

    let set_name = SetName::new(utils::long_name("a", 64));

    // register a new set
    {
        let res = utils::seat_register_set(seats, root, set_name.clone(), 0, u32::MAX, 1000, false);
        res.assert_success();
        seats.transfer_extra_deposit_to(root);
    }

    // register Alice in the Seats contract
    utils::seat_register_user(seats, alice);
    // note: Alice can now own tokens, but she will also be a vesting user.
    // This is the worst-case cost-wise.
    //
    // so we avoid taking the user extra funds for now.
    // seats.transfer_extra_deposit_to(root);

    // mint some seat tokens for Alice
    {
        utils::seat_force_mint(seats, root, alice, 2);

        // confirm that Alice has the SEAT tokens
        assert_eq!(utils::user_balance(seats, alice), 2);
    }

    // register Alice as a vesting user in the Seats contract
    {
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_name,
            alice,
            // 0%
            000_000_000_000,
        );
        res.assert_success();

        seats.transfer_extra_deposit_to(root);
    }
}

/// Creates many users that will participate in the system.
///
/// Stress test the minting functions to know the limits.
///
/// Conclusion: limiting each minting call to 100 users appeared relatively safe
/// according to the tests made.  
/// When using the system on the blockchain, it should take around 30s to mint
/// rewards for every 1000 vesting users. But a million vesting users would take
/// 8h at least.
///
/// This test is ignored by default because it's slow.
#[ignore]
#[test]
fn mint_gas_limit_test() {
    const USER_LEN: usize = 100;

    let (ref root, ref seats, ref _alice, ref _bob, ref _carol) = init(1, 0, None, 0);

    // create USER_LEN users
    let users: Vec<_> = (0..USER_LEN)
        .into_iter()
        .map(|i| {
            root.create_user(
                // format!("user{}", i).parse().unwrap(),
                utils::long_user(&format!("user{}", i)),
                parse_near!("10 kN"),
            )
        })
        .collect();

    let set_names: Vec<_> = (0..10)
        .map(|i| SetName::new(utils::long_name(&format!("set{}", i), 64)))
        .collect();

    // create 10 sets
    for set_name in &set_names {
        let res = utils::seat_register_set(seats, root, set_name.clone(), 0, u32::MAX, 1000, false);
        res.assert_success();
        seats.transfer_extra_deposit_to(root);
    }

    for (i, user) in users.iter().enumerate() {
        println!("{}", user.account_id());

        // register the users in the Seats contract
        utils::seat_register_user(seats, user);

        // enables the user as a vesting user
        //
        // each 10 users will be registered in a user set
        let _total_vesting_amount = utils::seat_register_vesting_user(
            seats,
            root,
            set_names[i / 10].clone(),
            user,
            // 10%
            010_000_000_000,
        );

        // mint some tokens to that user
        utils::seat_force_mint(
            seats,
            root,
            user,
            // 2 trillion.
            2 * TERA,
        );

        // take the extra deposits from the seats contract
        seats.transfer_extra_deposit_to(root);
    }

    // starts the minting operation
    {
        use seats::types::MintState;

        // initiate the minting procedure
        {
            // gives 0.3mN because a new timestamp is recorded in the contract
            root.transfer(seats.account_id(), parse_near!("0.3 mN"));

            let state = utils::start_mint(seats, root).unwrap_json();
            // the state progressed to Vesting
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 0,
                    user_offset: 0,
                }
            );
        };

        // the recommended value for the amount of users for each mint_step call
        // is 100~128.
        //
        // for the operations that follow, if the gas reaches ~204.855 TGas,
        // the call fails

        // continue the minting procedure (100 users/steps)
        {
            let state = utils::step_mint(seats, root, 100);
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 9,
                    user_offset: 1,
                }
            );
        };

        // continue the minting procedure (100 users/steps)
        {
            let state = utils::step_mint(seats, root, 100);
            assert_eq!(state, MintState::Standby);
        };
    }
}

/// Registers various sets and users on init.
#[ignore]
#[test]
fn init_with_test() {
    use seats::init::SetMembers;

    const USER_LEN: usize = 100;

    // create USER_LEN user_names
    let user_names: Vec<_> = (0..USER_LEN)
        .into_iter()
        .map(|i| utils::long_user(&format!("user{}", i)))
        .collect();

    let set_names: Vec<_> = (0..20)
        .map(|i| SetName::new(utils::long_name(&format!("set{}", i), 64)))
        .collect();

    // creates 20 sets, where each set has among 5 and 10 users.
    // for the sets that has 10 users, 5 will be shared with the set before
    // it, and 5 will be shared with the set after it
    let set_memebers = set_names
        .iter()
        .enumerate()
        .map(|(i, set_name)| SetMembers {
            set: set_name.clone(),
            // 0.01 SEAT * i
            reward: seats::types::RewardAmount(1000 * i as u128),
            start_delay_seconds: 0,
            expiration_delay_seconds: u32::MAX,
            members: user_names
                .iter()
                .skip(isize::max(i as isize * 10 - 5, 0) as usize)
                .take(15)
                .map(|user| {
                    (
                        user.clone(),
                        // 5%
                        seats::types::RewardPercentage(005_000_000_000),
                    )
                })
                .collect(),
            users_must_claim: false,
        })
        .collect::<Vec<_>>();

    let (ref root, ref seats, ref _alice, ref _bob, ref _carol) =
        init(1, 1, Some(set_memebers), parse_near!("2 N"));

    // starts the minting operation
    {
        use seats::types::MintState;

        // initiate the minting procedure
        {
            // gives 0.3mN because a new timestamp is recorded in the contract
            root.transfer(seats.account_id(), parse_near!("0.3 mN"));
            let state = utils::start_mint(seats, root).unwrap_json();
            // the state progressed to Vesting
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 0,
                    user_offset: 0,
                }
            );
        };

        // the recommended value for the amount of users for each mint_step call
        // is 100~128.

        // continue the minting procedure (100 users)
        {
            let state = utils::step_mint(seats, root, 100);
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 6,
                    user_offset: 4,
                }
            );
        };

        // continue the minting procedure (100 users)
        {
            let state = utils::step_mint(seats, root, 100);
            assert_eq!(state, MintState::Standby);
        };
    }

    // check user funds
    {
        for (i, user) in user_names.iter().enumerate() {
            let user = &root.create_user(user.clone(), parse_near!("10 kN"));

            let a = i / 10;
            let b = (i + 5) / 10;
            let b = if a == b { 0 } else { b };
            let tokens = (a + b) * 50;
            let info = utils::user_info(seats, user);
            dbg!(i);
            assert_eq!((tokens as u128, 0, tokens as u128), info);
        }
    }
}
