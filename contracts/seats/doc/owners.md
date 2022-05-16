###### Other Methods

Check all methods available to this contract on the main [API](./README.md) file.

# Owner Management
Methods:
- `add_owner`
- `remove_owner`
- `is_owner`
- `get_owners`

CRUD-like operations to add and remove owners, and to check if some account is an owner.

## Add Owner
Method: `add_owner`  
Description: Adds a new owner.  

###### Properties

- Changing (non-view).
- Must be called by an owner.

##### Parameters

- `owner_id`: string - The AccountId of the new owner being added.

Json example:
```json
{
    "owner_id": "owner2.stlb.testnet"
}
```

###### Return

Returns `true` if it's a newly added owner. Returns `false` if the owner was already added.

Json example:
```json
true
```

## Remove Owner
Method: `remove_owner`  
Description: removes an existing owner.  

###### Properties

- Changing (non-view).
- Must be called by an owner.

##### Parameters

- `owner_id`: string - The AccountId of the existing owner being removed.

Json example:
```json
{
    "owner_id": "owner2.stlb.testnet"
}
```

###### Return

Returns `true` if such owner was removed. Returns `false` if the owner wasn't added in the first place.

Json example:
```json
true
```

## Check If Is Owner
Method: `is_owner`  
Description: Checks if the given account is an owner.    

###### Properties

- Non-changing (view).

##### Parameters

- `owner_id`: string - The AccountId of the owner being checked.

Json example:
```json
{
    "owner_id": "owner2.stlb.testnet"
}
```

###### Return

Returns `true` if such account was an owner, and `false` otherwise.

Json example:
```json
true
```

## Get Owners
Method: `get_owners`  
Description: Get a list of the owners' account_ids.

###### Properties

- Non-changing (view).

##### Parameters

- `from_index`: optional string - Optional stringified 128-bit unsigned integer representing how many owners to skip. If `null`, then `0` owners are skiped - ie. a list starting from the first owner is created.
- `limit`: optional number - Optional 16-bit unsigned integer, representing how many owners to show. If `null`, then the maximum amount of owners is shown.

Json example:
```json
{
    "from_index": "2",
    "limit": 1
}
```

###### Return

Returns a list of account_ids.

Json example:
```json
["alice.near", "bob.near"]
```
