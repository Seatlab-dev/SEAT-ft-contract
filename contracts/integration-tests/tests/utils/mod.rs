use common::sim::{
    contract_ext::{Execution, View},
    ContractExt, WithAccount,
};
pub use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::{
    serde::{Deserialize, Serialize},
    serde_json::json,
    AccountId, Gas,
};
use near_sdk_sim::{ContractAccount, UserAccount};
use near_units::{parse_gas, parse_near};
use seats::types::MintState;
use seats::types::SetName;
use seats::{storage_costs as seats_costs, SeatsContract};
use stlb_seats_ft as seats;

pub const GAS: Gas = Gas(parse_gas!("300 Tgas") as u64);

pub struct LocalPin;

impl WithAccount<LocalPin> for SeatsContract {
    fn with_account(account_id: AccountId) -> Self {
        Self { account_id }
    }
}

/// Based on [`near_contract_standards::storage_management::StorageBalance`].
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    SEATS_WASM_BYTES => "../res/stlb_seats_ft.wasm",
}

/// Creates a user with a certain length.
pub fn long_user(s: &str) -> AccountId {
    let name = long_name(s, 64);
    AccountId::new_unchecked(name)
}

/// Creates a string with a certain length.
pub fn long_name(
    s: &str,
    len: u16,
) -> String {
    let len = len as usize - s.len();
    str::repeat("o", len) + s
}

pub fn dummy_metadata() -> near_contract_standards::fungible_token::metadata::FungibleTokenMetadata
{
    near_contract_standards::fungible_token::metadata::FungibleTokenMetadata {
        // required by validation
        spec: "ft-1.0.0".to_string(),
        name: "a".into(),
        symbol: "a".into(),
        icon: Some("a".into()),
        reference: None,
        reference_hash: None,
        decimals: 0,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup_seats(
    root: &UserAccount,
    contract_id: &str,
    mint_lock_duration_seconds: u32,
    start_timestamp_seconds: u32,
    metadata: Option<near_contract_standards::fungible_token::metadata::FungibleTokenMetadata>,
    set_members: Option<Vec<seats::init::SetMembers>>,
    extra_deposit: u128,
) -> ContractAccount<SeatsContract> {
    let seats = if let Some(set_members) = set_members {
        let metadata = metadata.unwrap();
        ContractAccount::<SeatsContract>::debug_json_deploy(
            root,
            contract_id,
            &SEATS_WASM_BYTES,
            "new_with",
            json!({
                "owner_id": root.account_id(),
                "metadata": metadata,
                "mint_lock_duration_seconds": mint_lock_duration_seconds,
                "start_timestamp_seconds": start_timestamp_seconds,
                "set_members": set_members
            }),
            GAS,
            seats_costs::CONTRACT + extra_deposit,
        )
    } else {
        let metadata = metadata.unwrap_or_else(seats::constant_init::metadata);
        ContractAccount::<SeatsContract>::debug_json_deploy(
            root,
            contract_id,
            &SEATS_WASM_BYTES,
            "new",
            json!({
                "owner_id": root.account_id(),
                "metadata": metadata,
                "mint_lock_duration_seconds": mint_lock_duration_seconds,
                "start_timestamp_seconds": start_timestamp_seconds
            }),
            GAS,
            seats_costs::CONTRACT + extra_deposit,
        )
    };

    seats.transfer_extra_deposit_to(root);
    seats
}

pub fn setup_seats_default(
    root: &UserAccount,
    contract_id: &str,
    network: seats::constant_init::Network,
    extra_deposit: u128,
) -> ContractAccount<SeatsContract> {
    let seats = ContractAccount::<SeatsContract>::debug_json_deploy(
        root,
        contract_id,
        &SEATS_WASM_BYTES,
        "new_const",
        json!({
            "owner_id": root.account_id(),
            "network": network
        }),
        GAS,
        seats_costs::CONTRACT + extra_deposit,
    );

    seats.transfer_extra_deposit_to(root);
    seats
}

pub fn seat_register_set(
    seats: &ContractAccount<SeatsContract>,
    root: &UserAccount,
    set: SetName,
    start_delay_seconds: u32,
    expiration_delay_seconds: u32,
    reward: u128,
    users_must_claim: bool,
) -> Execution<()> {
    seats.debug_json_call(
        root,
        "add_vesting_set",
        json!({
            "name": set,
            "start_delay_seconds": start_delay_seconds,
            "expiration_delay_seconds": expiration_delay_seconds,
            "reward": seats::types::RewardAmount(reward),
            "users_must_claim": users_must_claim
        }),
        GAS,
        seats_costs::USER_SET,
    )
}

pub fn seat_get_sets(seats: &ContractAccount<SeatsContract>) -> View<Vec<seats::types::SetName>> {
    seats.debug_json_view(
        "get_vesting_sets",
        json!(
        {
            "from_index": null,
            "limit": null
        }),
    )
}

pub fn seat_get_set(
    seats: &ContractAccount<SeatsContract>,
    set: &SetName,
) -> View<Option<seats::types::VestingUserSetInfo>> {
    seats.debug_json_view("get_vesting_set", json!({ "name": set }))
}

pub fn seat_register_user(
    seats: &ContractAccount<SeatsContract>,
    user: &UserAccount,
) {
    let balance_status: StorageBalance = seats
        .debug_json_call(
            user,
            "storage_deposit",
            json!({
                "account_id": null,
                "registration_only": null,
            }),
            GAS,
            seats_costs::user::BALANCE_REQUIREMENT,
        )
        .unwrap_json();
    assert_eq!(
        balance_status,
        StorageBalance {
            total: seats_costs::user::BALANCE_REQUIREMENT.into(),
            available: 0.into()
        }
    );
}

pub fn seat_unregister_user(
    seats: &ContractAccount<SeatsContract>,
    user: &UserAccount,
    force: Option<bool>,
) -> Execution<bool> {
    seats.debug_json_call(
        user,
        "storage_unregister",
        json!({ "force": force }),
        GAS,
        parse_near!("1 yN"),
    )
}

pub fn seat_register_vesting_user(
    seats: &ContractAccount<SeatsContract>,
    root: &UserAccount,
    set: SetName,
    user: &UserAccount,
    percentage: u64,
) -> Execution<()> {
    seats.debug_json_call::<()>(
        root,
        "add_vesting_user",
        json!({
            "set": set,
            "account_id": user.account_id(),
            "percentage": percentage.to_string(),
        }),
        GAS,
        // this deposit is more than needed, so the extra
        // is returned
        seats_costs::user::VESTING,
    )
}

pub fn seat_change_vesting_user(
    seats: &ContractAccount<SeatsContract>,
    root: &UserAccount,
    set: SetName,
    user: &UserAccount,
    new_percentage: u64,
) -> Execution<()> {
    seats.debug_json_call::<()>(
        root,
        "change_vesting_user",
        json!({
            "set": set,
            "account_id": user.account_id(),
            "new_percentage": new_percentage.to_string(),
        }),
        GAS,
        parse_near!("0 N"),
    )
}

pub fn seat_force_mint(
    seats: &ContractAccount<SeatsContract>,
    root: &UserAccount,
    user: &UserAccount,
    amount: u128,
) {
    let res = seats.debug_json_call(
        root,
        "force_mint",
        json!({
            "account_id": user.account_id(),
            "amount": amount.to_string()
        }),
        GAS,
        parse_near!("0 N"),
    );
    res.assert_success();
}

pub fn user_info(
    seats: &ContractAccount<SeatsContract>,
    user: &UserAccount,
) -> (u128, u128, u128) {
    let user_ = get_user(seats, user);

    let sets: Vec<SetName> = seats
        .debug_json_view(
            "get_vesting_sets",
            json!({
                "from_index": null,
                "limit": null
            }),
        )
        .unwrap_json();

    let mut reward = 0;
    for set in sets {
        let set_reward: seats::types::RewardAmount = seats
            .debug_json_view::<Option<seats::types::VestingUserSetInfo>>(
                "get_vesting_set",
                json!({ "name": set }),
            )
            .unwrap_json()
            .unwrap()
            .reward;

        let user_percentage: seats::types::RewardPercentage = seats
            .debug_json_view(
                "get_vesting_user",
                json!({
                    "set": set,
                    "account_id": user.account_id()
                }),
            )
            .unwrap_json();

        reward += user_percentage.to_reward(set_reward);
    }

    (user_.balance.0, user_.claim_balance.0, reward)
}

/// Views the storage balance of some ft contract.
pub fn user_balance<Contract>(
    ft: &ContractAccount<Contract>,
    user: &UserAccount,
) -> u128 {
    ft.debug_json_view::<U128>(
        "ft_balance_of",
        json!({
            "account_id": user.account_id()
        }),
    )
    .unwrap_json()
    .0
}

/// Views the storage balance of some ft contract.
pub fn get_user<Contract>(
    ft: &ContractAccount<Contract>,
    user: &UserAccount,
) -> seats::types::User {
    ft.debug_json_view(
        "get_user",
        json!({
            "account_id": user.account_id()
        }),
    )
    .unwrap_json()
}

/// Makes a transfer to the ft.
pub fn transfer(
    seats: &ContractAccount<SeatsContract>,
    sender: &UserAccount,
    receiver: &UserAccount,
    amount: u128,
) -> Execution<()> {
    seats.debug_json_call::<()>(
        sender,
        "ft_transfer",
        json!({
            "receiver_id": receiver.account_id(),
            "amount": amount.to_string(),
            "memo": null,
        }),
        GAS,
        parse_near!("1 yN"),
    )
}

/// Makes a transfer to the ft and a method call on the receiver.
///
/// Returns used token amount.
pub fn transfer_call<Contract>(
    ft: &ContractAccount<Contract>,
    sender: &UserAccount,
    receiver: &UserAccount,
    amount: u128,
) -> u128 {
    ft.debug_json_call::<U128>(
        sender,
        "ft_transfer_call",
        json!({
            "receiver_id": receiver.account_id(),
            "amount": amount.to_string(),
            "memo": null,
            "msg": "",
        }),
        GAS,
        parse_near!("1 yN"),
    )
    .unwrap_json()
    .0
}

/// Claims tokens.
pub fn claim(
    seats: &ContractAccount<SeatsContract>,
    user: &UserAccount,
) -> Execution<U128> {
    seats.debug_json_call(user, "claim", json!({}), GAS, parse_near!("0 N"))
}

pub fn total_supply(seats: &ContractAccount<SeatsContract>) -> u128 {
    seats
        .debug_json_view::<U128>("ft_total_supply", json!({}))
        .unwrap_json()
        .0
}

pub fn start_mint(
    seats: &ContractAccount<SeatsContract>,
    root: &UserAccount,
) -> Execution<MintState> {
    seats.debug_json_call(root, "start_mint", json!({}), GAS, parse_near!("0 N"))
}

pub fn step_mint(
    seats: &ContractAccount<SeatsContract>,
    caller: &UserAccount,
    limit: u16,
) -> MintState {
    seats
        .debug_json_call(
            caller,
            "step_mint",
            json!({
                "limit": limit,
            }),
            GAS,
            parse_near!("0 N"),
        )
        .unwrap_json()
}
