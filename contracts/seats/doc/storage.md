###### Other Methods

Check all methods available to this contract on the main [API](./README.md) file.

### Common Structures

See [NEP-145](https://nomicon.io/Standards/StorageManagement) for more info.

This is an interface from the near-contracts-standards. 

#### StorageBalance

This is a structure representing the balance status of a given account.  
It contains the fields:
- `total`: string - Stringfied 128-bit unsigned integer representing the total amount of yoctoNEAR that the user has deposited for him.
- `available`: string - Stringfied 128-bit unsigned integer representing the amount of yoctoNEAR that the user has deposited but is not being used by the contract, and which the user is free to withdraw.

### StorageBalanceBounds

This is a structure representing the minimum and maximum balance amounts that the contract may require from the user.

It contains the fields:  
- `min`: string - Stringfied 128-bit unsigned integer representing the amount of yoctoNEAR required to start using this contract at all (eg. to register with the contract). If a new contract user attaches `min` NEAR to a `storage_deposit` call, subsequent calls to `storage_balance_of` for this user must show their `total` equal to `min` and `available=0`.
- `max`: string - Optional stringfied 128-bit unsigned integer representing the maximum amount of yoctoNEAR that the contract may require from the user. If `null`, then there's no specific maximum balance amount that the contract may require from the user. If `max` enquals `min`, then the contract only charges for initial registration, and does not adjust per-user storage over time. Otherwise for some `max` amount, if the user has tried to deposit some amount higher than `max`, then the contract refunds that extra amount back to the user.


# Storage Management and User Registration
Methods:
- `storage_deposit`
- `storage_withdraw`
- `storage_unregister`
- `storage_balance_bounds`
- `storage_balance_of`

The Storage Management interface is used when users are getting registered to own fungible tokens. A fixed value is paid for registration, and during unregistration, the account being unregistered receives the same amount back from the contract.

###### Notes

- Owners may configure sets and set members, and when doing this, the Storage Management is _not_ used. Owners must pay fixed bonds when making calls that would result a storage cost increase related to those actions.

## Storage Deposit
Method: `storage_deposit`
Description: Receives an attached deposit of NEAR for a given account. This is how an account is registered into the contract so it's able to own tokens.

###### Properties

- Changing (non-view).
- Is payable.
- Panics during minting and migrations.

###### Parameters
- `account_id`: optional string - The account_id that is receiving the deposit, that is being registered. If `null`, then the account in question is assumed to be the caller himself.
- `registration_only`: optional boolean - This value is always assumed to be `true` by this contract, in which case any extra paid deposit is returned back to the caller.

Json example:
```json
{
    "account_id": "alice.near", 
    "registration_only": null
}
```

###### Return

Returns the [StorageBalance](#storagebalance) showing updated balances.

Json example:
```json
{
  "total": "2000000000000000000000",
  "available": "0"
}
```


## Storage Withdraw
Method: `storage_withdraw`
Description: Withdraw specified amount of `available` NEAR for predecessor account. As this contract always leave zero as `available` amounts for accounts, this method will either panic or show the account_id balance information.

###### Properties

- Changing (non-view).
- Is payable (one yocto).
- Panics during minting and migrations.

###### Parameters
- `amount`: optional string - Optional stringfied 128-bit unsigned integers representing the amount of yoctoNEAR tokens to be withdrew from the user's `available` balance. If `null`, then the full `available` balance is withdrew. 


Json example:
```json
{
    "amount": "0"
}
```

###### Note

As this contract always have the `available` balance set to zero, then this function will always panic except when `amount` is set to zero, in which case no transfer is made back to the caller and the user balance information is returned.

###### Return

Returns the [StorageBalance](#storagebalance) showing updated balances.

Json example:
```json
{
  "total": "2000000000000000000000",
  "available": "0"
}
```

## Storage Unregister
Method: `storage_unregister`
Description: Unregisters the predecessor account.

###### Properties
- Changing (non-view).
- Is payable (one yocto).
- Panics during minting and migrations.

###### Parameters
- `force`: optional boolean - Whether the removal should be forced. If `true`, then assets from the user (token amount) are removed or burned. Otherwise if `null` then `false` is assumed, and in the `false` case then the function will fail if the user being unregistered still own assets.

Json example:
```json
{
    "force": true
}
```

###### Notes

- If the user being removed is still registered as a vesting user, then that user won't get any token rewards during vesting user minting operations - ie. that user is skipped during rewards, and the tokens it would gain are effectively burned.

###### Return

Returns `true` iff the account was successfully unregistered; returns `false` iff the account was not registered before.

Json example:
```json
true
```

## Storage Balance Bounds
Method: `storage_balance_bounds`
Description: Get the contract's setting for minimum and maximum of storage balance requirements for the users.

###### Properties
- Nonchanging (view).


###### Parameters

Has no parameters.

###### Return

Returns the contract's settings for the [StorageBalanceBounds](#storagebalancebounds).

Json example:
```json
{
    "min": "2000000000000000000000",
    "max": "2000000000000000000000"
}
```


## Storage Balance Of User
Method: `storage_balance_of`
Description: Gets the storage balance information of a given account_id.

###### Parameters

- `account_id`: string - Account_id that the balance information is being queried from.

Json example:
```json
{
    "account_id": "alice.near"
}
```

###### Return

Optionally returns the [StorageBalance](#storagebalance) information of the given user. Returns `null` if the account is not registered.

Json example:
```json
{
  "total": "2000000000000000000000",
  "available": "0"
}
```
