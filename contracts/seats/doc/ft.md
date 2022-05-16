###### Other Methods

Check all methods available to this contract on the main [API](./README.md) file.

### Common Structures

#### Fungible Token Interface

See [NEP-141](https://nomicon.io/Standards/Tokens/FungibleToken/Core) for more info.


# Fungible Token
Methods:
- `ft_transfer`
- `ft_transfer_call`
- `ft_total_supply`
- `ft_balance_of`

## Transfer
Method: `ft_transfer`  
Description: Simple transfer to a receiver.

###### Properties

- Changing (non-view).
- Is payable (one yocto).
- Must be called by the assets' owner.
- Panics during minting and migrations.

##### Parameters

- `receiver_id`: string - AccountId of the receiver.
- `amount`: string - Stringfied 128-bit unsigned integer representing the amount of [raw SEAT](./README.md#raw-seat-token) tokens being transferred. 
- `memo`: optional string - Used by use cases that may benefit from indexing or providing information for a transfer.

Json example:
```json
{
  "receiver_id": "bob.near",
  "amount": "712345",
  "memo": null
}
```

##### Return

Has no return.


## Transfer Call
Method: `ft_transfer_call`  
Description: Transfer tokens and then calls the `ft_on_transfer` method on the receiver contract - which acknowledges or denies the transfer - and then makes a callback on the `ft_resolve_transfer` method back to the fungible token contract. This workflow can, for example, be used to "attach" tokens as a "deposit" in a call to a receiver contract.

###### Properties

- Changing (non-view).
- Is payable (one yocto).
- Must be called by the assets' owner.
- Panics during minting and migrations.

##### Parameters

- `receiver_id`: string - AccountId of the receiver.
- `amount`: string - Stringfied 128-bit unsigned integer representing the amount of [raw SEAT](./README.md#raw-seat-token) tokens being transferred. 
- `memo`: optional string - Used by use cases that may benefit from indexing or providing information for a transfer.
- `msg`: string - Sent as the `msg` parameter on the `receiver_id`'s `ft_on_transfer` method. Can send arbitrary information that the receiver may require. 

Json example:
```json
{
  "receiver_id": "bob.near",
  "amount": "712345",
  "memo": null,
  "msg": ""
}
```

###### Notes

- The `receiver_id`, as a contract, must implement the `ft_on_transfer` method, which indicates whether the transfer was accepted or not. Please check NEP-141 for more information.


##### Return

Returns a stringfied 128-bit unsigned integer representation of the amount of [raw SEAT](./README.md#raw-seat-token) tokens that were used from the sender.

Json example:
```json
"712345"
```

## Total Supply
Method: `ft_total_supply`  
Description: Gets the total supply of [raw SEAT](./README.md#raw-seat-token) tokens.

###### Properties

- Non-changing (view).

##### Parameters

Has no parameters.

##### Return

Returns a stringfied 128-bit unsigned integer representation of the total amount of [raw SEAT](./README.md#raw-seat-token) tokens that exist in the contract.

Json example:
```json
"712345"
```

## User Balance
Method: `ft_balance_of`  
Description: Get the balance of [raw SEAT](./README.md#raw-seat-token) tokens of a user.

###### Properties

- Non-changing (view).

##### Parameters

- `account_id`: string - The account_id of the user being queried.

##### Return

Returns a stringfied 128-bit unsigned integer representation of the amount of [raw SEAT](./README.md#raw-seat-token) tokens that the user owns.

Json example:
```json
"712345"
```

