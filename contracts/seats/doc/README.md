## SEAT Fungible Token

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

### Common Structures

#### Raw SEAT Token

A "raw" SEAT token is a number that is usually represented as a stringfied 128-bit unsigned integer. Sometimes it can also be represented as a stringfied 64-bit unsigned integer.  
As SEAT tokens has 5 decimal places, then the first 5 less significant unit places from the the "raw" are used to represent those decimal values. In other words, `1` SEAT token equals to `10^5` "raw" SEAT tokens.
For example, the amount `"712345"` of "raw" SEAT tokens represent `7.12345` SEAT tokens.


### Interface
Methods:
- [init](./init.md)
    - `new`
    - `new_const`
    - `new_with`
- [owners](./owners.md)
    - `add_owner`
    - `remove_owner`
    - `is_owner`
    - `assert_owner`
    - `get_owners`
- [storage/registration](./storage.md)
    - `storage_deposit`
    - `storage_withdraw`
    - `storage_unregister`
    - `storage_balance_bounds`
    - `storage_balance_of`
- [ft](./ft.md)
    - `ft_transfer`
    - `ft_transfer_call`
    - `ft_total_supply`
    - `ft_balance_of`
    - `ft_metadata`
- [mint](./mint.md)
    - `change_start_timestamp`
    - `get_start_timestamp`
    - `force_mint`
    - `start_mint`
    - `step_mint`
    - `force_end_mint`
    - `get_mint_state`
- [vesting sets](./sets.md)
    - `add_vesting_set`
    - `change_vesting_set`
    - `remove_vesting_set`
    - `get_vesting_set`
    - `get_vesting_sets`
    - `get_vesting_set_users`
    - `get_vesting_total`
- [vesting users](./users.md)
    - `add_vesting_user`
    - `get_vesting_user`
    - `change_vesting_user`
    - `remove_vesting_user`
    - `claim`
- [misc](./misc.md)
    - `force_start_migration`
    - `force_end_migration`
    - `version`