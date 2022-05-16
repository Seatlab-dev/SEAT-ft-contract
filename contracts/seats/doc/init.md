###### Other Methods

Check all methods available to this contract on the main [API](./README.md) file.

### Common Structures

#### Fungible Token Metadata
See [NEP-148](https://nomicon.io/Standards/Tokens/FungibleToken/Metadata) for more info.

This is a structure from the near-contracts-standards. 

It contains the fields:
- `spec`: string - Should be exactly the string `"ft-1.0.0"`, as this is required by the current metadata self-validation.
- `name`: string - The human-readable name of the token.
- `symbol`: string - The abbreviation, like wETH or AMPL.
- `icon`: optional string - A small image associated with this token. Must be a data URL, to help consumers display it quickly while protecting user data. For a "square" symbol, one can use the URL-escaped version of `data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg'><rect width='50' height='50' /></svg>`.
- `reference`: optional string - A link to a valid JSON file containing various keys offering supplementary details on the token. Eg. `/ipfs/QmdmQXB2mzChmMeKY47C43LxUdg1NDJ5MWcKMKxDu7RgQm`, `https://example.com/token.json`. If the information given in this document conflicts with the on-chain attributes, the values in reference shall be considered the source of truth.
- `reference_hash`: optional string - The base64-encoded sha256 hash of the JSON file contained in the reference field. This is to guard against off-chain tampering.
- `decimals`: number - 8-bit unsigned integer representing the amount of decimals used by the token unit. Used in frontends to show the proper significant digits of a token. This concept is explained well in this [OpenZeppelin post](https://docs.openzeppelin.com/contracts/3.x/erc20#a-note-on-decimals).

#### Set Member

This is a structure representing some configurations of a vesting/user set.

It contains the fields:
- `set`: string - The set name.
- `reward`: string - A stringified 64-bits unsigned integer number representing how many [raw SEAT](./README.md#raw-seat-token) tokens, at maximum, will be distributed to the members of the set. 
- `start_delay_seconds`: number - 32-bit unsigned integer, representing for how many seconds after the contract's `start_date` this set won't mint rewards.
- `expiration_delay_seconds`: number - 32-bit unsigned integer, for how many seconds, after `start_delay_seconds`, this set will still mint rewards.
- `users_must_claim`: boolean - Whether members must still claim their token rewards, or if they get directly rewarded of their tokens.
- `members`: object list: List of members registered on this set. Each member requires two values:
    - `[0]`: string - The AccountId of the member. 
    - `[1]`: string - This is a stringified 64-bit unsigned integer number representing the "reward percentage". This is a percentage-like stringfied integer, but with more integer (mantissa, characteristic) places so that the calculations have a higher precision. 100% is represented as "100000000000", whereas 1% is represented as "001000000000". Lower values are percentages below 1%.


# Init
Methods:

- `new`
- `new_with`
- `new_const`

## New
Method: `new`  
Description: Initializes the SEAT FT contract.

###### Properties

- Changing (non-view).
- For initialization.
- Is payable.

##### Parameters

- `owner_id`: string - The contract owner.
- `metadata`: object - FT Metadata. See [fungible-token-metadata](#fungible-token-metadata) for more info.
- `mint_lock_duration_seconds`: number - 32-bit unsigned integer. After starting a minting operation, how many seconds should must the next minting operation wait for before getting started.
- `start_timestamp_seconds`: number - 32-bit unsigned integer. Unix timestamp before which the contract should be locked.

Json example:
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
        "decimals": 5
    },
    "mint_lock_duration_seconds": 60,
    "start_timestamp_seconds": 1641006000
}
```

###### Notes

- `1641006000` is the unix timestamp of `2022-01-01 00:00:00`.

###### Return

Has no return.


## New (with sets)
Method: `new_with`
Description: Initializes the SEAT FT contract with some initial vesting/user sets.

###### Properties

- Changing (non-view).
- For initialization.
- Is payable.

##### Parameters

The same as [new](#new), with additionally:

- `set_members`: object list - List for vesting sets information to be added. For each element, see the [set member](#set-member) for more information.

Json example:
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

###### Notes

- Aside from the initial `3N` on SEAT FT contract account, an extra `1N` should be attached when deploying the contract that would contain many sets and members on initialization. This value could be lower, and for an exact attachment requirement, please check the `storage_deposit`, `add_vesting_set` and `add_vesting_user` methods and their attachment requirements, and make an attachment accordingly to the amount of sets and user registrations.
- `start_timestamp_seconds` requires a unix timestamp, in which case `1641006000` is the unix timestamp of `2022-01-01 00:00:00`.

###### Return

Has no return.


## New (constants)
Method: `new_const`
Description: Initializes the SEAT FT contract with some pre-determined initial configuration.

###### Properties

- Changing (non-view).
- For initialization.
- Is payable.

##### Parameters

Similar to [new_with](#new-with-sets), except that most parameters are constants defined by the contract. Is still requires the parameters:

- `owner_id`: string - The contract owner.
- `network`: string - The network that the contract is being deployed to. Possible values: `"Testnet"` | `"Mainnet"`.
- `start_timestamp_seconds`: optional number - Optional 32-bit unsigned integer. Unix timestamp before which the contract should be locked. If `null`, defaults to 1 day in the future when `network` is `"Mainnet"`, or 1 minute in the future when `network` is `Testnet`.


Json example:
```json
{
    "owner_id": "owner.stlb.testnet", 
    "network": "Testnet",
    "start_timestamp_seconds": 1641006000,
}
```

###### Notes

- Aside from the initial `3N` on SEAT FT contract account, an extra `1N` should be attached when deploying the contract that would contain many sets and members on initialization.
- `start_timestamp_seconds` requires a unix timestamp, in which case `1641006000` is the unix timestamp of `2022-01-01 00:00:00`.

###### Return

Has no return.
