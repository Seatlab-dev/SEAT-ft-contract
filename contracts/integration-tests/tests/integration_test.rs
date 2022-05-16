#![allow(clippy::needless_borrow)]
#![allow(clippy::zero_prefixed_literal)]

use common::sim::{ContractExt, ExecutionExt};
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

/// 10_000_000 SEAT tokens.
const TERA: u128 = 1_000_000_000_000;

fn init(
    mint_lock_duration_seconds: u32,
    start_timestamp_seconds: u32,
    metadata: Option<near_contract_standards::fungible_token::metadata::FungibleTokenMetadata>,
    set_members: Option<Vec<seats::init::SetMembers>>,
    extra_deposit: u128,
) -> (
    UserAccount,
    ContractAccount<SeatsContract>,
    UserAccount,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);
    let seats_id: AccountId = "seats".parse().unwrap();

    let seats = utils::setup_seats(
        &root,
        &seats_id.to_string(),
        mint_lock_duration_seconds,
        start_timestamp_seconds,
        metadata,
        set_members,
        extra_deposit,
    );

    let alice = root.create_user("alice".parse().unwrap(), parse_near!("10 kN"));
    let bob = root.create_user("bob".parse().unwrap(), parse_near!("10 kN"));
    let carol = root.create_user("carol".parse().unwrap(), parse_near!("10 kN"));

    (root, seats, alice, bob, carol)
}

/// Registers users and mint rewards for them.
#[test]
fn transfer_test() {
    let (ref root, ref seats, ref alice, ref bob, ref _carol) = init(0, 0, None, None, 0);

    // register Alice in the Seats contract
    utils::seat_register_user(seats, alice);

    // mint some seat tokens for Alice
    {
        utils::seat_force_mint(seats, root, alice, 2);

        // confirm that Alice has the SEAT tokens
        assert_eq!(utils::user_info(seats, alice), (2, 0, 0));
    }

    // register Bob in the Seats contract
    utils::seat_register_user(seats, bob);

    // make a simple transfer
    {
        utils::transfer(seats, alice, bob, 1).assert_success();
    }

    #[allow(clippy::identity_op)]
    {
        assert_eq!(utils::user_info(seats, alice), (1, 0, 0));
        assert_eq!(utils::user_info(seats, bob), (1, 0, 0));
    }
}

/// Registers users and mint rewards for them.
#[test]
fn mint_test() {
    let (ref root, ref seats, ref alice, ref bob, ref carol) = init(0, 0, None, None, 0);

    let set_a = &SetName::new("set-a".to_string());
    let set_b = &SetName::new("set-b".to_string());

    // register sets
    //
    // Bob will participate in set a and b, and Carol will participate in set b.
    {
        let res = utils::seat_register_set(seats, root, set_a.clone(), 0, u32::MAX, 1000, false);
        res.assert_success();

        let res = utils::seat_register_set(seats, root, set_b.clone(), 0, u32::MAX, 2000, false);
        res.assert_success();
    }

    // register Alice in the Seats contract
    utils::seat_register_user(seats, alice);

    // mint some seat tokens for Alice
    {
        utils::seat_force_mint(seats, root, alice, 2);

        // confirm that Alice has the SEAT tokens
        assert_eq!(utils::user_info(seats, alice), (2, 0, 0));
    }

    // register Bob in the Seats contract
    utils::seat_register_user(seats, bob);

    // register Bob as a vesting user in the Seats contract
    {
        // set a
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_a.clone(),
            bob,
            // 100%
            100_000_000_000,
        );
        res.assert_success();

        // set b
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_b.clone(),
            bob,
            // 50%
            050_000_000_000,
        );
        res.assert_success();

        // confirm that Bob is a vesting user
        assert_eq!(
            utils::user_info(seats, bob),
            (
                0,
                0,
                // set a
                1000 * 100 / 100
                // set b
                + 2000 * 50 / 100
            )
        );
    }

    // register Carol in the Seats contract
    utils::seat_register_user(seats, carol);

    // mint some seat tokens for Carol
    {
        utils::seat_force_mint(seats, root, carol, 2);

        // confirm that Carol has the seat tokens
        assert_eq!(utils::user_info(seats, carol), (2, 0, 0));
    }

    // register Carol as a vesting user in the Seats contract
    {
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_b.clone(),
            carol,
            // 50%
            050_000_000_000,
        );
        res.assert_success();

        // confirm that Carol is a vesting user
        assert_eq!(
            utils::user_info(seats, carol),
            (
                2,
                0,
                // set b
                2000 * 50 / 100
            )
        );
    }

    // user info (seat tokens, vesting):
    // Alice: (2, 0, 0)
    // Bob: (0, 0, 2000)
    // Carol: (2, 0, 1000)
    //

    // minting
    //
    // Alice: 0 = 0.
    // Bob: 100 * 1000 / 100 + 50 * 2000 / 100 = 2000.
    // Carol: 50 * 2000 * / 100 = 1000.
    {
        use seats::types::MintState;

        // check that the minting hasn't started yet
        {
            let state: MintState = seats
                .debug_json_view("get_mint_state", json!({}))
                .unwrap_json();
            assert_eq!(state, MintState::Standby);
        }

        // initiate the minting procedure
        {
            let state = utils::start_mint(seats, root).unwrap_json();
            // the state progressed to Vesting
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 0,
                    user_offset: 0,
                }
            );
        }

        // continue the minting procedure (0/3 users)
        {
            let state = utils::step_mint(seats, root, 0);
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 0,
                    user_offset: 0,
                }
            );
        }

        // continue the minting procedure (1/3 users)
        {
            let state = utils::step_mint(seats, root, 1);
            // one vesting user is still missing
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 0,
                    user_offset: 1,
                }
            );
        }

        // continue the minting procedure (1/3 users)
        // (progresses the set to the next one)
        {
            let state = utils::step_mint(seats, root, 1);
            // both Bob and Carol got their vesting rewards,
            // so the state has progressed to the next state/phase
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 1,
                    user_offset: 0,
                }
            );
        }

        // continue the minting procedure (2/3 users)
        {
            let state = utils::step_mint(seats, root, 1);
            // both Bob and Carol got their vesting rewards,
            // so the state has progressed to the next state/phase
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 1,
                    user_offset: 1,
                }
            );
        }

        // continue the minting procedure (3/3 users)
        // (marked to move into the next set, which doesn't exist)
        {
            let state = utils::step_mint(seats, root, 1);
            // both Bob and Carol got their vesting rewards,
            // so the state has progressed to the next state/phase
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 1,
                    user_offset: 2,
                }
            );
        }

        // continue the minting procedure (3/3 users)
        {
            let state = utils::step_mint(seats, root, 1);
            // both Bob and Carol got their vesting rewards,
            // so the state has progressed to the next state/phase
            assert_eq!(state, MintState::Standby);
        }
    }

    #[allow(clippy::identity_op)]
    {
        assert_eq!(utils::user_info(seats, alice), (2, 0, 0));
        assert_eq!(
            utils::user_info(seats, bob),
            (0 + 1000 + 50 * 2000 / 100, 0, 1000 + 50 * 2000 / 100)
        );
        assert_eq!(
            utils::user_info(seats, carol),
            (2 + 50 * 2000 / 100, 0, 50 * 2000 / 100)
        );

        let set_a_info = utils::seat_get_set(seats, set_a).unwrap_json().unwrap();
        let set_b_info = utils::seat_get_set(seats, set_b).unwrap_json().unwrap();
        //
        assert_eq!(set_a_info.total_rewarded.0, 1000);
        assert_eq!(set_a_info.generation, 1);
        assert_eq!(set_b_info.total_rewarded.0, 2000);
        assert_eq!(set_b_info.generation, 1);
    }
}

/// Check mint timelocks: start_timestmap and after-mint timelocks.
#[test]
fn mint_timelock_test() {
    let (ref root, ref seats, ref alice, ref _bob, ref _carol) = init(100, 100, None, None, 0);

    let set_a = &SetName::new("set-a".to_string());

    // register sets
    //
    // Alice will participate in set-a.
    {
        let res = utils::seat_register_set(seats, root, set_a.clone(), 0, u32::MAX, 1000, false);
        res.assert_success();
    }

    // register Alice in the Seats contract
    utils::seat_register_user(seats, alice);

    // register Alice as a vesting user in the Seats contract
    {
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_a.clone(),
            alice,
            // 50%
            050_000_000_000,
        );
        res.assert_success();

        // confirm that Alice is a vesting user
        assert_eq!(
            utils::user_info(seats, alice),
            (
                0,
                0,
                // set a
                1000 * 50 / 100
            )
        );
    }

    // user info (seat tokens, vesting):
    // Alice: (3, 0, 500)

    // minting
    //
    // total rewards: 500.
    // Alice: 1000 * 50/100 = 500.
    {
        use seats::types::MintState;

        // initiate the minting procedure
        // (fails, start_timestmap not yet passed)
        {
            let res = utils::start_mint(seats, root);

            res.assert_failure_with(r"start_mint not yet enabled, \d+ nanoseconds remaining");
        }

        // advances some blocks
        {
            let mut runtime = root.borrow_runtime_mut();
            runtime.produce_blocks(100).unwrap();
        }

        // initiate the minting procedure
        {
            let state = utils::start_mint(seats, root).unwrap_json();
            // the state progressed to Vesting
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 0,
                    user_offset: 0,
                }
            );
        }

        // continue the minting procedure (1/1 user)
        {
            let state = utils::step_mint(seats, root, 2);
            assert_eq!(state, MintState::Standby);
        }
    }

    #[allow(clippy::identity_op)]
    {
        assert_eq!(utils::user_info(seats, alice), (500, 0, 500));

        let set_a_info = utils::seat_get_set(seats, set_a).unwrap_json().unwrap();
        //
        assert_eq!(set_a_info.total_rewarded.0, 500);
        assert_eq!(set_a_info.generation, 1);
    }

    // user info (seat tokens, vesting):
    // Alice: (500, 0, 500)

    // minting
    //
    // total rewards: 500.
    // Alice: 1000 * 50/100 = 500.
    {
        use seats::types::MintState;

        // initiate the minting procedure
        {
            let res = utils::start_mint(seats, root);
            res.assert_failure_with(r"start_mint locked, \d+ nanoseconds remaining");

            // the state is still in Standby
            let state = seats
                .debug_json_view::<MintState>("get_mint_state", json!({}))
                .unwrap_json();
            assert_eq!(state, MintState::Standby);
        }

        // advances some blocks
        {
            let mut runtime = root.borrow_runtime_mut();
            runtime.produce_blocks(100).unwrap();
        }

        // initiate the minting procedure
        {
            let state = utils::start_mint(seats, root).unwrap_json();

            // the state progressed to Vesting
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 0,
                    user_offset: 0,
                }
            );
        }

        // continue the minting procedure (2/2 users)
        {
            let state = utils::step_mint(seats, root, 2);
            assert_eq!(state, MintState::Standby);
        }
    }

    #[allow(clippy::identity_op)]
    {
        assert_eq!(utils::user_info(seats, alice), (500 + 500, 0, 500));

        let set_a_info = utils::seat_get_set(seats, set_a).unwrap_json().unwrap();
        //
        assert_eq!(set_a_info.total_rewarded.0, 1000);
        assert_eq!(set_a_info.generation, 2);
    }
}

/// Check user set timelocks: start_date and expiration_date timelocks.
#[test]
fn user_set_timelock_test() {
    let (ref root, ref seats, ref alice, ref bob, ref _carol) = init(0, 0, None, None, 0);

    let set_a = &SetName::new("set-a".to_string());
    let set_b = &SetName::new("set-b".to_string());

    // register sets
    //
    // Alice will participate in set-a, and Bob on set-b.
    //
    // time:    0 --- 200 --- 300 --- 500 --- 600
    // set-a:          * ------------- *
    // set-b:                  * ------------- *
    // active sets: 00 1111111 2222222 1111111 0
    // mint points:  *     *   *           *   *
    //               A     B   C           D   E
    {
        let res = utils::seat_register_set(seats, root, set_a.clone(), 200, 300, 1, false);
        res.assert_success();

        let res = utils::seat_register_set(seats, root, set_b.clone(), 300, 300, 10, false);
        res.assert_success();
    }

    // register Alice in the Seats contract
    utils::seat_register_user(seats, alice);

    // register Alice as a vesting user in the Seats contract
    {
        // set a
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_a.clone(),
            alice,
            // 100%
            100_000_000_000,
        );
        res.assert_success();

        // confirm that Alice is a vesting user
        assert_eq!(utils::user_info(seats, alice), (0, 0, 1));
    }

    // register Bob in the Seats contract
    utils::seat_register_user(seats, bob);

    // register Bob as a vesting user in the Seats contract
    {
        // set b
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_b.clone(),
            bob,
            // 100%
            100_000_000_000,
        );
        res.assert_success();

        // confirm that Bob is a vesting user
        assert_eq!(utils::user_info(seats, bob), (0, 0, 10));
    }

    let now = |root: &UserAccount| {
        seats::types::Timestamp::from(root.borrow_runtime().current_block().block_timestamp)
            .seconds_part()
    };

    // user info (seat tokens, vesting):
    // Alice: (0, 0, 1)
    // Bob: (0, 0, 10)

    // minting (A)
    //
    // total rewards: 0.
    {
        use seats::types::MintState;

        // confirms we are in minting (A)
        assert!(now(root) < 200);

        // initiate and make steps on the minting procedure
        // (no set is active)
        {
            utils::start_mint(seats, root).assert_success();
            while !matches!(utils::step_mint(seats, root, 2), MintState::Standby) {}

            // confirm user assets
            assert_eq!(utils::user_info(seats, alice), (0, 0, 1));
            assert_eq!(utils::user_info(seats, bob), (0, 0, 10));

            let set_a_info = utils::seat_get_set(seats, set_a).unwrap_json().unwrap();
            let set_b_info = utils::seat_get_set(seats, set_b).unwrap_json().unwrap();
            //
            assert_eq!(set_a_info.total_rewarded.0, 0);
            assert_eq!(set_a_info.generation, 0);
            assert_eq!(set_b_info.total_rewarded.0, 0);
            assert_eq!(set_b_info.generation, 0);
        }

        // advances some blocks
        {
            let mut runtime = root.borrow_runtime_mut();
            runtime.produce_blocks(200).unwrap();
        }

        // confirms we are in minting (B)
        assert!((200..300).contains(&now(root)));

        // initiate and make steps on the minting procedure
        // (set-a is active)
        {
            utils::start_mint(seats, root).assert_success();
            while !matches!(utils::step_mint(seats, root, 2), MintState::Standby) {}

            // confirm user assets
            assert_eq!(utils::user_info(seats, alice), (1, 0, 1));
            assert_eq!(utils::user_info(seats, bob), (0, 0, 10));

            let set_a_info = utils::seat_get_set(seats, set_a).unwrap_json().unwrap();
            let set_b_info = utils::seat_get_set(seats, set_b).unwrap_json().unwrap();
            //
            assert_eq!(set_a_info.total_rewarded.0, 1);
            assert_eq!(set_a_info.generation, 1);
            assert_eq!(set_b_info.total_rewarded.0, 0);
            assert_eq!(set_b_info.generation, 0);
        }

        let set_b_start_date = utils::seat_get_set(seats, set_b)
            .unwrap_json()
            .unwrap()
            .start_date;

        let curr = now(root);
        // advances some blocks
        {
            let mut runtime = root.borrow_runtime_mut();
            runtime
                .produce_blocks((set_b_start_date.seconds_part() - curr) as u64)
                .unwrap();
        }

        // confirms we are in minting (C)
        assert_eq!(now(root), set_b_start_date.seconds_part());

        // initiate and make steps on the minting procedure
        // (set-a and set-b are active)
        {
            utils::start_mint(seats, root).assert_success();
            while !matches!(utils::step_mint(seats, root, 2), MintState::Standby) {}

            // confirm user assets
            assert_eq!(utils::user_info(seats, alice), (2, 0, 1));
            assert_eq!(utils::user_info(seats, bob), (10, 0, 10));

            let set_a_info = utils::seat_get_set(seats, set_a).unwrap_json().unwrap();
            let set_b_info = utils::seat_get_set(seats, set_b).unwrap_json().unwrap();
            //
            assert_eq!(set_a_info.total_rewarded.0, 2);
            assert_eq!(set_a_info.generation, 2);
            assert_eq!(set_b_info.total_rewarded.0, 10);
            assert_eq!(set_b_info.generation, 1);
        }

        // advances some blocks
        {
            let mut runtime = root.borrow_runtime_mut();
            runtime.produce_blocks(200).unwrap();
        }

        // confirms we are in minting (D)
        assert!((500..600).contains(&now(root)));

        // initiate and make steps on the minting procedure
        // (set B is active)
        {
            utils::start_mint(seats, root).assert_success();
            while !matches!(utils::step_mint(seats, root, 2), MintState::Standby) {}

            // confirm user assets
            assert_eq!(utils::user_info(seats, alice), (2, 0, 1));
            assert_eq!(utils::user_info(seats, bob), (20, 0, 10));

            let set_a_info = utils::seat_get_set(seats, set_a).unwrap_json().unwrap();
            let set_b_info = utils::seat_get_set(seats, set_b).unwrap_json().unwrap();
            //
            assert_eq!(set_a_info.total_rewarded.0, 2);
            assert_eq!(set_a_info.generation, 2);
            assert_eq!(set_b_info.total_rewarded.0, 20);
            assert_eq!(set_b_info.generation, 2);
        }

        let set_b_expiration_date = utils::seat_get_set(seats, set_b)
            .unwrap_json()
            .unwrap()
            .expiration_date;

        let curr = now(root);
        // advances some blocks
        {
            let mut runtime = root.borrow_runtime_mut();
            runtime
                .produce_blocks((set_b_expiration_date.seconds_part() - curr) as u64)
                .unwrap();
        }

        // confirms we are in minting (D)
        assert_eq!(now(root), set_b_expiration_date.seconds_part());

        // initiate and make steps on the minting procedure
        // (no set is no longer active)
        {
            utils::start_mint(seats, root).assert_success();
            while !matches!(utils::step_mint(seats, root, 2), MintState::Standby) {}

            // confirm user assets
            assert_eq!(utils::user_info(seats, alice), (2, 0, 1));
            assert_eq!(utils::user_info(seats, bob), (20, 0, 10));

            let set_a_info = utils::seat_get_set(seats, set_a).unwrap_json().unwrap();
            let set_b_info = utils::seat_get_set(seats, set_b).unwrap_json().unwrap();
            //
            assert_eq!(set_a_info.total_rewarded.0, 2);
            assert_eq!(set_a_info.generation, 2);
            assert_eq!(set_b_info.total_rewarded.0, 20);
            assert_eq!(set_b_info.generation, 2);
        }
    }
}

#[test]
fn test_mint_percentages_high_init() {
    use seats::init::SetMembers;
    use seats::types::RewardPercentage;

    let set_a = &SetName::new("set-a".to_string());

    let alice_as_member = SetMembers {
        set: set_a.clone(),
        reward: seats::types::RewardAmount(TERA),
        start_delay_seconds: 0,
        expiration_delay_seconds: u32::MAX,
        users_must_claim: false,
        members: vec![(
            "alice".parse().unwrap(),
            // higher than RewardPercentage::MAX
            RewardPercentage(100_000_000_001),
        )],
    };

    let res = std::panic::catch_unwind(|| {
        init(
            0,
            0,
            Some(utils::dummy_metadata()),
            Some(vec![alice_as_member.clone()]),
            parse_near!("1 N"),
        )
    })
    .err()
    .unwrap()
    .downcast::<String>()
    .unwrap();

    assert!(res.contains("Smart contract panicked: percentage 100000000001 is too high"));
}

#[test]
fn test_mint_percentages_high_add_or_change() {
    use seats::init::SetMembers;
    use seats::types::RewardPercentage;

    let set_a = &SetName::new("set-a".to_string());
    let set_b = &SetName::new("set-b".to_string());

    let alice_as_member = SetMembers {
        set: set_a.clone(),
        reward: seats::types::RewardAmount(TERA),
        start_delay_seconds: 0,
        expiration_delay_seconds: u32::MAX,
        users_must_claim: false,
        members: vec![(
            "alice".parse().unwrap(),
            // at the RewardPercentage::MAX
            RewardPercentage(100_000_000_000),
        )],
    };

    let bob_as_member = SetMembers {
        set: set_b.clone(),
        reward: seats::types::RewardAmount(TERA),
        start_delay_seconds: 0,
        expiration_delay_seconds: u32::MAX,
        users_must_claim: false,
        members: vec![("bob".parse().unwrap(), RewardPercentage(0))],
    };

    let (ref root, ref seats, ref _alice, ref bob, ref _carol) = init(
        0,
        0,
        Some(utils::dummy_metadata()),
        Some(vec![alice_as_member, bob_as_member]),
        parse_near!("1 N"),
    );

    // registers bob in set-a, which only allows for a zero percentage for new
    // members
    {
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_a.clone(),
            bob,
            // really low percentage
            000_000_000_001,
        );
        res.assert_failure_with("Smart contract panicked: percentage 100000000001 is too high");
    }

    // successfully register (with zero percentage),
    // but fails when increasing that percentage
    {
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_a.clone(),
            bob,
            // 0%
            000_000_000_000,
        );
        res.assert_success();

        let res = utils::seat_change_vesting_user(
            seats,
            root,
            set_a.clone(),
            bob,
            // really low percentage
            000_000_000_001,
        );
        res.assert_failure_with("Smart contract panicked: percentage 100000000001 is too high");
    }
}

#[test]
fn constant_init_testnet_values_test() {
    use seats::{constant_init::Network, types::Timestamp};

    const NETWORK: Network = Network::Testnet;

    let root = init_simulator(None);
    let seats_id: AccountId = "seats".parse().unwrap();

    let day_zero = Timestamp::from(root.borrow_runtime().cur_block.block_timestamp)
        + Timestamp::from_seconds(1);

    let start_date =
        day_zero + Timestamp::from_seconds(seats::constant_init::start_timestamp_seconds(&NETWORK));

    let seats =
        &utils::setup_seats_default(&root, &seats_id.to_string(), NETWORK, parse_near!("1 N"));

    let sets = utils::seat_get_sets(seats).unwrap_json();
    let sets = sets
        .iter()
        .map(|set| {
            (
                set.0.as_str(),
                utils::seat_get_set(seats, set).unwrap_json().unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let init_values = seats::constant_init::set_members(&NETWORK);
    let init_values = init_values
        .iter()
        .map(|set| {
            (
                set.set.0.as_str(),
                seats::types::VestingUserSetInfo {
                    reward: set.reward,
                    total_rewarded: seats::types::RewardAmount::default(),
                    generation: u32::default(),
                    last_mint_timestamp: Timestamp::default(),
                    start_date: start_date + Timestamp::from_seconds(set.start_delay_seconds),
                    expiration_date: start_date
                        + Timestamp::from_seconds(set.start_delay_seconds)
                        + Timestamp::from_seconds(set.expiration_delay_seconds),
                    total_user_percentages: set
                        .members
                        .iter()
                        .map(|member| member.1 .0)
                        .sum::<u64>()
                        .into(),
                    users_must_claim: set.users_must_claim,
                },
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(sets, init_values);
}

#[test]
fn constant_init_mainnet_values_test() {
    use seats::{constant_init::Network, types::Timestamp};

    const NETWORK: Network = Network::Mainnet;

    let root = init_simulator(None);
    let seats_id: AccountId = "seats".parse().unwrap();

    let day_zero = Timestamp::from(root.borrow_runtime().cur_block.block_timestamp)
        + Timestamp::from_seconds(1);

    let start_date =
        day_zero + Timestamp::from_seconds(seats::constant_init::start_timestamp_seconds(&NETWORK));

    let seats =
        &utils::setup_seats_default(&root, &seats_id.to_string(), NETWORK, parse_near!("1 N"));

    let sets = utils::seat_get_sets(seats).unwrap_json();
    let sets = sets
        .iter()
        .map(|set| {
            (
                set.0.as_str(),
                utils::seat_get_set(seats, set).unwrap_json().unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let init_values = seats::constant_init::set_members(&NETWORK);
    let init_values = init_values
        .iter()
        .map(|set| {
            (
                set.set.0.as_str(),
                seats::types::VestingUserSetInfo {
                    reward: set.reward,
                    total_rewarded: seats::types::RewardAmount::default(),
                    generation: u32::default(),
                    last_mint_timestamp: Timestamp::default(),
                    start_date: start_date + Timestamp::from_seconds(set.start_delay_seconds),
                    expiration_date: start_date
                        + Timestamp::from_seconds(set.start_delay_seconds)
                        + Timestamp::from_seconds(set.expiration_delay_seconds),
                    total_user_percentages: set
                        .members
                        .iter()
                        .map(|member| member.1 .0)
                        .sum::<u64>()
                        .into(),
                    users_must_claim: set.users_must_claim,
                },
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(sets, init_values);
}

/// Registers users, mint rewards for them and test their caliming of
/// those rewards.
#[test]
fn test_claim() {
    let (ref root, ref seats, ref alice, ref _bob, ref _carol) = init(0, 0, None, None, 0);

    let set_a = &SetName::new("set-a".to_string());

    // register sets
    //
    // Alice will participate in set a.
    {
        let res = utils::seat_register_set(seats, root, set_a.clone(), 0, u32::MAX, 1000, true);
        res.assert_success();
    }

    // register Alice in the Seats contract
    utils::seat_register_user(seats, alice);

    // register Alice as a vesting user in the Seats contract
    {
        let res = utils::seat_register_vesting_user(
            seats,
            root,
            set_a.clone(),
            alice,
            // 50%
            050_000_000_000,
        );
        res.assert_success();

        // confirm that Alice is a vesting user
        assert_eq!(
            utils::user_info(seats, alice),
            (
                0,
                0,
                // set b
                1000 * 50 / 100
            )
        );
    }

    // user info (seat tokens, vesting):
    // Alice: (2, 0, 1000)
    //

    // minting
    //
    // Alice: 0 = 0.
    // Alice: 50 * 2000 * / 100 = 1000. (must claim)
    {
        use seats::types::MintState;

        // check that the minting hasn't started yet
        {
            let state: MintState = seats
                .debug_json_view("get_mint_state", json!({}))
                .unwrap_json();
            assert_eq!(state, MintState::Standby);
        }

        // initiate the minting procedure
        {
            let state = utils::start_mint(seats, root).unwrap_json();
            // the state progressed to Vesting
            assert_eq!(
                state,
                MintState::Vesting {
                    set_offset: 0,
                    user_offset: 0,
                }
            );
        }

        // continue the minting procedure (1/1 users)
        {
            let state = utils::step_mint(seats, root, 100);
            assert_eq!(state, MintState::Standby);
        }
    }

    #[allow(clippy::identity_op)]
    {
        assert_eq!(
            utils::user_info(seats, alice),
            (0, 50 * 1000 / 100, 50 * 1000 / 100)
        );
    }

    // Alice claims her tokens
    {
        let amount = utils::claim(seats, alice);
        assert_eq!(amount.unwrap_json().0, 50 * 1000 / 100);
        assert_eq!(
            utils::user_info(seats, alice),
            (
                50 * 1000 / 100,
                // no longer has any balance for claiming
                0,
                50 * 1000 / 100
            )
        );
    }
}

/// Registers users and mint rewards for them.
#[test]
fn mint_for_removed_user_test() {
    let res = std::panic::catch_unwind(|| {
        let (ref root, ref seats, ref alice, ref bob, ref _carol) = init(0, 0, None, None, 0);

        let set_a = &SetName::new("set-a".to_string());

        // register sets
        //
        // Alice and Bob will participate in set-a.
        {
            let res =
                utils::seat_register_set(seats, root, set_a.clone(), 0, u32::MAX, 1000, false);
            res.assert_success();
        }

        // register Alice in the Seats contract
        utils::seat_register_user(seats, alice);

        // register Bob in the Seats contract
        utils::seat_register_user(seats, bob);

        // register Alice and Bob as vesting users in the Seats contract
        {
            // set a
            let res = utils::seat_register_vesting_user(
                seats,
                root,
                set_a.clone(),
                alice,
                // 50%
                050_000_000_000,
            );
            res.assert_success();

            let res = utils::seat_register_vesting_user(
                seats,
                root,
                set_a.clone(),
                bob,
                // 50%
                050_000_000_000,
            );
            res.assert_success();

            // confirm that they are vesting users
            assert_eq!(
                utils::user_info(seats, alice),
                (
                    0,
                    0,
                    // set a
                    1000 * 50 / 100
                )
            );
            assert_eq!(
                utils::user_info(seats, bob),
                (
                    0,
                    0,
                    // set a
                    1000 * 50 / 100
                )
            );
        }

        // unregister Bob from the contract
        // (Bob will still be a vesting user)
        {
            let is_removed = utils::seat_unregister_user(seats, bob, Some(false)).unwrap_json();
            assert!(is_removed);
        }

        // user info (seat tokens, vesting):
        // Alice: (2, 0, 500)
        // Bob: (-, -, 500)
        //

        // minting
        //
        // Alice: 50 * 1000 / 100 = 500.
        // Bob: 0.
        {
            use seats::types::MintState;

            // check that the minting hasn't started yet
            {
                let state: MintState = seats
                    .debug_json_view("get_mint_state", json!({}))
                    .unwrap_json();
                assert_eq!(state, MintState::Standby);
            }

            // initiate the minting procedure
            {
                let state = utils::start_mint(seats, root).unwrap_json();
                // the state progressed to Vesting
                assert_eq!(
                    state,
                    MintState::Vesting {
                        set_offset: 0,
                        user_offset: 0,
                    }
                );
            }

            // continue the minting procedure (1/1 users)
            {
                let state = utils::step_mint(seats, root, 100);
                assert_eq!(state, MintState::Standby);
            }
        }

        #[allow(clippy::identity_op)]
        {
            // the minting procedure got executed, skipping bob
            assert_eq!(
                utils::user_info(seats, alice),
                (50 * 1000 / 100, 0, 50 * 1000 / 100)
            );

            // the contract didn't mint any tokens for bob
            assert_eq!(utils::total_supply(seats), 500);

            // this fails because bob is not registered
            utils::user_info(seats, bob)
        }
    });
    let res = res.err().unwrap().downcast::<String>().unwrap();
    assert!(res.contains("The account bob is not registered"));
}
