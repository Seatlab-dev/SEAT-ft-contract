#!/bin/bash

# This is a testnet example of how to deploy and test the SEAT FT contract.
# You can use this to guide your testing. 
#
# Please don't try execute this script file directly because some functions, 
# like the `start_mint` one, requires a minute to have passed from the deployment.

set -ex

# create the development account and deploy the contract to it
near dev-deploy --wasmFile=contracts/res/stlb_seats_ft.wasm

# copy the env to $CONTRACT_NAME
source neardev/dev-account.env
export SEAT="$CONTRACT_NAME"
export OWNER="owner.$SEAT"
export ALICE="alice.$SEAT"
export BOB="bob.$SEAT"

# create some accounts
near create-account "$OWNER" --masterAccount "$SEAT" --initialBalance "8"
near create-account "$ALICE" --masterAccount "$SEAT" --initialBalance "4"
near create-account "$BOB" --masterAccount "$SEAT" --initialBalance "4"

# initialize the SEAT FT contract
METHOD='new_const'
ARGS='{"owner_id": "'"$OWNER"'", "network": "Testnet"}'
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId "$OWNER" \
    --depositYocto "0" \
    --gas 300000000000000
# note: $SEAT already has enough funds, so it doesn't need extra deposits
# if it needed more funds, you could transfer from another account with `near send`.

# check the set names that got created
METHOD='get_vesting_sets'
ARGS='{}'
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"

# check the "seed" set (that it has a reward, it's start_date, etc)
METHOD='get_vesting_set'
ARGS='{"name": "seed"}'
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"

# register Alice into the contract
METHOD="storage_deposit"
ARGS='{"account_id": "'"$ALICE"'"}'
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId "$OWNER" \
    --depositYocto "4000000000000000000000" \
    --gas "300000000000000"

# register Alice as a member on the "seed" set
# she will get 10% of the set reward
METHOD="add_vesting_user"
ARGS='{"set": "seed", "account_id": "'"$ALICE"'", "percentage": "010000000000"}'
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "5000000000000000000000" \
    --gas "300000000000000"


# confirm that the "seed" set has it's total percentage to 10%
METHOD='get_vesting_set'
ARGS='{"name": "seed"}'
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"
# reward: 8805302000
# total_user_percentages: '10000000000' which is 10% of it's capacity


# confirm that Alice is registered in the "seed" set
METHOD='get_vesting_set_users'
ARGS='{"name": "seed"}'
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"
# [ [ 'alice.dev-xxxxxxxxxxxxx-xxxxxxxxxxxxxx', '10000000000' ] ]

# Note: need to wait for 1 minute to have passed before calling
# the start_mint procedure.
sleep 60

# start and progress the daily reward minting operation
METHOD="start_mint"
ARGS='{}'
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "0" \
    --gas "300000000000000"
# { t: 'Vesting', c: { set_offset: 0, user_offset: 0 } }

# progress the daily reward
METHOD="step_mint"
ARGS='{}'
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId $OWNER \
    --depositYocto "0" \
    --gas "300000000000000"
# { t: 'Standby' }

# check Alice's funds
METHOD='ft_balance_of'
ARGS='{"account_id": "'"$ALICE"'"}'
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"
# `0`
# note: this is correct as Alice must still claim her new tokens

# check Alice's complete user info
METHOD='get_user'
ARGS='{"account_id": "'"$ALICE"'"}'
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"
# { balance: '0', claim_balance: '880530200' }
# note: the claim_balance is correct as Alice now has 10% of 
# the total set reward of 8805302000

# makes Alice claim her minted tokens
METHOD="claim"
ARGS='{}'
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId "$ALICE" \
    --depositYocto "0" \
    --gas "300000000000000"
# '880530200'
# note: 8805.302 SEAT tokens claimed.

# check Alice's funds again
METHOD='ft_balance_of'
ARGS='{"account_id": "'"$ALICE"'"}'
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"
# `880530200`

# register Bob into the contract
METHOD="storage_deposit"
ARGS='{"account_id": "'"$BOB"'"}'
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId "$OWNER" \
    --depositYocto "4000000000000000000000" \
    --gas "300000000000000"

# have Alice transfer 1 SEAT token to Bob
METHOD="ft_transfer"
ARGS='{"receiver_id": "'"$BOB"'", "amount": "100000"}'
near call \
    "$SEAT" \
    "$METHOD" \
    "$ARGS" \
    --accountId "$ALICE" \
    --depositYocto "1" \
    --gas "300000000000000"
# note: as the token has 5 decimal places, that "amount" was necessary

# confirm Bob receivement
METHOD='ft_balance_of'
ARGS='{"account_id": "'"$BOB"'"}'
near view \
    "$SEAT" \
    "$METHOD" \
    "$ARGS"
# '100000'