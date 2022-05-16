# User Story: Overview

###### Note: Reading JSON to vars

For the examples in this file, the json representation of the arguments are always shown separately from the bash scripts, but the later always assume that the `ARGS` env var will contain that json.

For example, for a given method call, suppose that there is this json representation of the arguments:
```json
{
    "some_key": "some value"
}
```

The script of that method call will assume that the variable `ARGS` contains that json, such as in:
```bash
read -r -d '' ARGS << END_OF_JSON
{
    "some_key": "some value"
}
END_OF_JSON
```

So for every method call, please assume that this is how the `ARGS` is constructed, and it's always from the json representation of the arguments.

## Init

This shows how the contracts could or should be initialized.

Assumptions:
- Ownership over the user `owner.stlb.testnet`, which will manage the contracts.
- Ownership over a cleared `seat.stlb.testnet`, which will be the SEAT Fungible Token contract.
  - This account should have `3N`+ deposited to it.
  - This account should have `2N`+ deposited to it.
- Ownership over users: `alice.testnet`, `bob.testnet` and `carol.testnet`.
- All contracts are in the same commit version of this document (you can take the files from the releases page).
- For scripts that execute, that they do at the root of the repository.

##### SEAT

The arguments in json:
```json
{
    "owner_id": "owner.stlb.testnet", 
    "metadata": {
        "spec": "ft-1.0.0", 
        "name": "SeatlabNFT Fungible Token", 
        "symbol": "SEAT", 
        "icon": null, 
        "reference": null, 
        "reference_hash": null, 
        "decimals": 12
    },
    "mint_lock_duration_seconds": 60,
    "start_timestamp_seconds": 1641006000
}
```

Deployment of the SEAT FT contract:
```bash
SEAT="seat.stlb.testnet"
INIT_FUNCTION="new"
near deploy \
    --wasmFile "contracts/res/stlb_seats_ft.wasm" \
    --contractName "$SEAT" \
    --initFunction "$INIT_FUNCTION" \
    --initArgs "$ARGS"
```


This will initialize the contract with a basic configuration.  
In this state, the contract still doesn't have any users registered, nor any tokens minted.

###### Notes

- `1641006000` is the unix timestamp of `2022-01-01 00:00:00`.


###### Initializing User Sets and Users on Init

Alternatively, it's possible to to use the `new_with` method on deployment and have some user sets registered and configured, including having users registered on the contracts and on the sets.

The arguments in json:
```json
{
    "owner_id": "owner.stlb.testnet", 
    "metadata": {
        "spec": "ft-1.0.0", 
        "name": "SeatlabNFT Fungible Token", 
        "symbol": "SEAT", 
        "icon": null, 
        "reference": null, 
        "reference_hash": null, 
        "decimals": 12
    },
    "mint_lock_duration_seconds": 60,
    "start_timestamp_seconds": 1641006000,
    "set_members": [
        {
            "set": "x",
            "reward": "1000",
            "start_delay_seconds": 60,
            "expiration_delay_seconds": 60,
            "users_must_claim": false,
            "members": [
                ["alice.near", "040000000000"],
                ["bob.near", "030000000000"],
                ["carol.near", "030000000000"]
            ]
        },
        {
            "set": "y",
            "reward": "5000",
            "start_delay_seconds": 0,
            "expiration_delay_seconds": 4294967295,
            "users_must_claim": true,
            "members": [
                ["alice.near", "100000000000"]
            ]
        },
        {
            "set": "z",
            "reward": "4000",
            "start_delay_seconds": 60,
            "expiration_delay_seconds": 0,
            "users_must_claim": false,
            "members": []
        }
    ]
}
```

Deployment of the SEAT FT contract:
```bash
SEAT="seat.stlb.testnet"
INIT_FUNCTION="new_with"
near deploy \
    --wasmFile "contracts/res/stlb_seats_ft.wasm" \
    --contractName "$SEAT" \
    --initFunction "$INIT_FUNCTION" \
    --initArgs "$ARGS" \
    --initGas "300000000000000" \
    --initDeposit "1.000000000000000000000000"
```

Notes:
- Aside from the initial `3N` on the account, an extra `1N` should be attached when deploying the contract that would contain many sets and members on initialization. This value could be lower, and for an exact attachment requirement, please check the `storage_deposit`, `add_vesting_set` and `add_vesting_user` methods and their attachment requirements, and make an attachment accingly to the sets and users that you are registering.
- `start_timestamp_seconds` requires a unix timestamp, in which case `1641006000` is the unix timestamp of `2022-01-01 00:00:00`.
- `set_members[i].start_delay_seconds` requires some duration value in seconds. The contract will calculate a start timestamp from the `start_timestamp_seconds` timestamp plus that start delay in seconds.
- `set_members[i].expiration_delay_seconds` requires some duration value in seconds. The contract will calculate an expiration timestamp from the start timestamp above plus that expiration delay in seconds. That is, the delay is applied _after_ the set's start timestamp. On the example above, the last set is expired immediately after it's start timestamp.
- `set_members[i].members[j][1]` require a reward percentage-like stringfied integer, but with more integer (mantissa, characteristic) places so that the calculations have a higher precision. 100% is represented as "100000000000", whereas 1% is represented as "001000000000". Lower values are percentages below 1%.
- `users_must_claim` chooses whether the set directly gives the minted tokens to it's members, or if the members must later claim their tokens in order to completely receive them.


This will initialize the contract with a basic configuration, including registering the user sets and their users. 

###### Automatically Initializing User Sets and Users on Init

Alternatively, it's possible to to use the `new_const` method on deployment and have some user sets registered and configured automatically, including having users registered on the contracts and on the sets.

The arguments in json:
```json
{
    "owner_id": "owner.stlb.testnet", 
    "network": "Testnet",
    "start_timestamp_seconds": 1641006000
}
```

Deployment of the SEAT FT contract:
```bash
SEAT="seat.stlb.testnet"
INIT_FUNCTION="new_const"
near deploy \
    --wasmFile "contracts/res/stlb_seats_ft.wasm" \
    --contractName "$SEAT" \
    --initFunction "$INIT_FUNCTION" \
    --initArgs "$ARGS" \
    --initGas "300000000000000" \
    --initDeposit "1.000000000000000000000000"
```

Notes:
- Aside from the initial `3N` on the account, an extra `1N` should be attached when deploying the contract that would contain many sets and members on initialization. This value could be lower, and for an exact attachment requirement, please check the `storage_deposit`, `add_vesting_set` and `add_vesting_user` methods and their attachment requirements, and make an attachment accordingly to the sets and users that you are registering.
- The `network` can be either `"Testnet"` | `"Mainnet"`.
- `start_timestamp_seconds` requires an optional unix timestamp, in which case `1641006000` is the unix timestamp of `2022-01-01 00:00:00`. Please check the API documentation for more information.

This will initialize the contract with a basic configuration, including registering the user sets and their users. 

## Basic Usage

### User Registration

The SEAT contract only knows about it's owner/manager. It still doesn't have any other user registered on it - except if initialized with the `new_with`.  

##### SEAT

After being registered, the user `alice.testnet` can:
- Own SEAT tokens on the SEAT contract. 
- Have vesting percentages in user sets, which is configured by the SEAT owner.

To register the user `alice.testnet`, the method `storage_deposit` can be used. 

The arguments in json:
```json
{
    "account_id": "alice.testnet", 
    "registration_only": null
}
```

```bash
METHOD="storage_deposit"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId "alice.testnet" \
    --depositYocto "4000000000000000000000" \
    --gas "300000000000000"
```

###### Notes

- Alice can register herself, but other users can also register her.
  - If Alice is registering herself, `"account_id"` can be null/omitted.
- Deposit requirement of `4 mN`.

### User Sets

Different user sets can be registered in the contract. They have a name/title, such as `"x"`, `"y"` or `"z"`; they also define an amount of token units as rewards, for the recurrent minting, that will be distributed among it's members; and finally, they have member list inside of them.

Each user inside of a set must define the percentage-like value of the rewards that they get.  

###### Note

- The percentage-like value for all users are verified when getting inserted or changed in the contract. If they are above 100%, or if the sum of users percentages for a set goes above 100%, the call making the change fails.
- The "User Sets" are also often called "Vesting Sets" by the contract.

#### User Set Management

##### Add User Set

To add a new user set, the method `add_vesting_set` can be used.

The arguments in json:
```json
{
    "name": "x", 
    "reward": "1000",
    "start_delay_seconds": 60,
    "expiration_delay_seconds": 60,
    "users_must_claim": false
}
```

```bash
METHOD="add_vesting_set"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "6000000000000000000000" \
    --gas "300000000000000"
```

###### Notes

- Must be called by the OWNER.
- Deposit requirement of `6 mN`.
- The `reward` is how many [raw SEAT](../seats/doc/README.md#raw-seat-token) tokens, at maximum, will be distributed to the members of the set.  
- `start_delay_seconds` requires some duration value in seconds. The contract will calculate a start timestamp from the contract's start date timestamp plus that start delay in seconds.
- `expiration_delay_seconds` requires some duration value in seconds. The contract will calculate an expiration timestamp from the set's start timestamp plus that expiration delay in seconds. That is, the delay is applied _after_ the set's start timestamp.
- `users_must_claim` chooses whether the set directly gives the minted tokens to it's members, or if the members must later claim their tokens in order to completely receive them.

##### Change User Set

To change the reward amount of an existing user set, the method `change_vesting_set` can be used.

The arguments in json:
```json
{
    "name": "x", 
    "new_reward": "2000",
    "new_start_date": "1641006000",
    "new_expiration_date": "1672542000",
    "new_users_must_claim": "1672542000"
}
```

```bash
METHOD="change_vesting_set"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "0" \
    --gas "300000000000000"
```

###### Notes

- Must be called by the OWNER.
- The `new_reward` is how much SEAT tokens will be distributed to the members of the set.
- `new_start_date` requires a unix timestamp, in which case `"1641006000"` is the unix timestamp of `2022-01-01 00:00:00`.
- `new_expiration_date` requires a unix timestamp, in which case `"1672542000"` is the unix timestamp of `2023-01-01 00:00:00`.
- `new_users_must_claim` chooses whether the set directly gives the minted tokens to it's members, or if the members must later claim their tokens in order to completely receive them.


##### Remove User Set

To remove an existing user set, the method `remove_vesting_set` can be used.

The arguments in json:
```json
{
    "name": "x", 
    "force": false
}
```

```bash
METHOD="remove_vesting_set"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "0" \
    --gas "300000000000000"
```

###### Notes

- Must be called by the OWNER.
- If the set still has members on it, then the members will be unregistered if `force` is set to `true`. Otherwise, if it still has members but if `force` is `false` or `null`, then the user set removal will fail. Otherwise if it has no members, then the set removal should succed.
- When the set removal succeds, then `6 mN` is returned to the caller; this was paid when
the user set was created.

##### View Methods


###### Get User Sets Names

To get a list of the user sets names, the method `get_vesting_sets` can be used.

The arguments in json:
```json
{
    "from_index": null, 
    "limit": null
}
```

```bash
METHOD="get_vesting_sets"
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" 
```

Returns a list of the sets names.

###### Get Users from a Set

To get a list of the users from a set, the method `get_vesting_set_users` can be used.

The arguments in json:
```json
{
    "name": "x",
    "from_index": null, 
    "limit": null
}
```

```bash
METHOD="get_vesting_set_users"
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" 
```

Returns a list of the users inside of that set.

###### Get Set Information

To the rewards and timestamps information from a set, the method `get_vesting_set` can be used.

The arguments in json:
```json
{
    "name": "x"
}
```

```bash
METHOD="get_vesting_set"
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" 
```

Returns `null` if the set doesn't exist, or otherwise returns a json in the following format:
```json
{
    "reward": "100",
    "start_date": "1641006000",
    "expiration_date": "1672542000",
    "total_user_percentages": "050000000000",
    "users_must_claim": false
}
```

Notes:
- `start_date` is a unix timestamp, in which case `"1641006000"` is the unix timestamp of `2022-01-01 00:00:00`.
- `expiration_date` is a unix timestamp, in which case `"1672542000"` is the unix timestamp of `2023-01-01 00:00:00`.
- `total_user_percentages` is a reward percentage-like stringfied integer, but with more integer (mantissa, characteristic) places so that the calculations have a higher precision. 100% is represented as "100000000000", whereas 1% is represented as "001000000000". Lower values are percentages below 1%.
- `users_must_claim` chooses whether the set directly gives the minted tokens to it's members, or if the members must later claim their tokens in order to completely receive them.

###### Get the Sum of the Rewards from All Sets

To get the sum of the rewards from all sets, the `get_vesting_total` method can be used.

The arguments in json:
```json
{}
```

```bash
METHOD="get_vesting_set_reward"
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" 
```

Returns a stringfied value of the total amount of tokens that is rewarded from all sets.

#### User Management

##### Add User in a Set

To add a new user in a set, the method `add_vesting_user` can be used.

The arguments in json:
```json
{
    "set": "x", 
    "account_id": "alice.near",
    "percentage": "050000000000"
}
```

```bash
METHOD="add_vesting_user"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "5000000000000000000000" \
    --gas "300000000000000"
```

###### Notes

- Must be called by the OWNER.
- Deposit requirement of `5 mN`.
- The `percentage` is a reward percentage-like stringfied integer, but with more integer (mantissa, characteristic) places so that the calculations have a higher precision. 100% is represented as "100000000000", whereas 1% is represented as "001000000000". Lower values are percentages below 1%.

#### Get User from a Set

To get the percentage of a user in a set, the method `get_vesting_user` can be used.

The arguments in json:
```json
{
    "set": "x", 
    "account_id": "alice.near"
}
```

```bash
METHOD="get_vesting_user"
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"
```

Returns the stringfied user's reward percentage in the set. That percentage is a reward percentage-like stringfied integer, but with more integer (mantissa, characteristic) places so that the calculations have a higher precision. 100% is represented as "100000000000", whereas 1% is represented as "001000000000" (without the left padding zeroes). Lower values are percentages below 1%.


###### Notes

- If the user is not in the set, then `"0"` is returned.

##### Change User from a Set

To change the percentage of a new user in a set, the method `change_vesting_user` can be used.

The arguments in json:
```json
{
    "set": "x", 
    "account_id": "alice.near",
    "new_percentage": "020000000000"
}
```

```bash
METHOD="change_vesting_user"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "0" \
    --gas "300000000000000"
```

###### Notes

- Must be called by the OWNER.
- The `new_percentage` is a reward percentage-like stringfied integer, but with more integer (mantissa, characteristic) places so that the calculations have a higher precision. 100% is represented as "100000000000", whereas 1% is represented as "001000000000". Lower values are percentages below 1%.

##### Remove User from a Set

To remove a new user from a set, the method `remove_vesting_user` can be used.

The arguments in json:
```json
{
    "set": "x", 
    "account_id": "alice.near"
}
```

```bash
METHOD="remove_vesting_user"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "0" \
    --gas "300000000000000"
```

###### Notes

- Must be called by the OWNER.
- `5 mN` is returned to the caller; this was paid when the user was inserted into the set.


### Mint SEAT Tokens

The SEAT contract offers two ways to mint tokens: 

1. Forced minting, which creates and deposits SEAT tokens to a specific user. 
2. Distributing minting, which creates and distribute SEAT tokens to the vesting users inside of the user sets, according to the sets `reward` setting.

##### Check Balance

To check Alice's SEAT balance, the `ft_balance_of` can be used.

The arguments in json:
```json
{
    "account_id": "alice.testnet"
}
```

```bash
METHOD="ft_balance_of"
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"
```

#### Force Mint

To forcefully mint 100 SEAT tokens and deposit it to Alice, the `force_mint` method can be used.

The arguments in json:
```json
{
    "account_id": "alice.testnet", 
    "amount": "100",
    "must_claim": false
}
```

```bash
METHOD="force_mint"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "0" \
    --gas "300000000000000"
```

###### Notes

- Must be called by the OWNER.
- `users_must_claim` can be `null`, and it chooses whether the minted tokens is given directly to the user, or if he must later claim the minted tokens in order to completely receive them.

#### Distributing Mint

Note: To see more about vesting users in vesting sets, please check it's sections.

The minting procedures have 2 different states, which are: 
1. `"Standby"`, when no minting is being done, or when the last minting procedure has finished.
2. `"Vesting"`, when the minting procedure is giving SEAT tokens to vesting users, or when the minting procedure has just started.

The actual json representation of the state is "adjacently tagged", with the following format: the `t` field to repersent the tag as a string, such as `"Standby"` | `"Vesting"`; and the `c` field to represent the content that each tag has. 
- The `"Standby"` variant has no content, so the `"c"` field will be missing. 
- The `"Vesting"`'s content has the fields `"set_offset"` to a number (indicating which set is beign worked on), `"user_offset"` to a number (indicating which user from the set is being worked on). 

To start the minting procedure that will distribute SEAT tokens to vesting users, according to their sets settings and each user percentages, the `start_mint` method can be used.

The arguments in json:
```json
{}
```

```bash
METHOD="start_mint"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "0" \
    --gas "300000000000000"
```

Return example:
```json
{
  "t": "Vesting",
  "c": {
    "set_offset": 0,
    "user_offset": 0,
  }
}
```

This return example indicates that the current state is `"Vesting"`, and also indicates the current progress of the minting operation (to work on the first set, first user of the set).

###### Notes

- Must be called by the OWNER.
- The fungible token methods, and others, will be locked until the distributing minting procedure finishes. This is because the minting procedure needs to have fixed values for correct calculations.
- This makes the distributing minting state go from `"Standby"` to `"Vesting"`.

##### Continue The Distributing Mint Procedure

The distributing minting procedure have 2 different states:

The minting procedure goes from `"Standby"`, when it still hasn't started, to `"Vesting"`, when it starts rewarding vesting users. After those users have been rewarded, the state goes back to `"Standby"`, indicating that the procedure has finished.

To progress on the minting procedure for 100 users, the `step_mint` method can be used.

The arguments in json:
```json
{
    "limit": "100"
}
```

```bash
METHOD="step_mint"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "0" \
    --gas "300000000000000"
```

###### Return Example (1)

```json
{
  "t": "Vesting",
  "c": {
    "set_offset": 1,
    "user_offset": 2,
  }
}
```

The return example above indicates that the third user of the  second set is being worked on.

###### Return Example (2)

The return example above indicates that minting procedure has finished.

```json
{
  "t": "Standby"
}
```


###### Notes

- The method returns the state in which the minting procedure ended up in. 
    - While the returned state is not `"Standby"`, the `step_mint` method should be called again.

### Claim Tokens

A user may need to claim tokens that were minted for him. For this, the `claim` function can be used.


The arguments in json:
```json
{}
```

```bash
METHOD="claim"
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $ALICE \
    --depositYocto "0" \
    --gas "300000000000000"
```

Return example:
```json
'880530200'
```

###### Notes

- The caller claims for his own account.
- The returned value represent the amount of raw SEAT tokens claimed. In this example, `8805,302` SEAT tokens were claimed.
 