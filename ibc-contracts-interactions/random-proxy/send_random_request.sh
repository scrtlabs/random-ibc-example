#!/usr/bin/env bash

# Set the contract address
contract="secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg"

# Check if the contract is deployed
echo "Checking if contract $contract is deployed..."
while ! secretcli q wasm code $contract &> /dev/null; do
    sleep 1
done
echo "Contract $contract is deployed!"

# Check if IBC channel-0 is open
echo "Checking if IBC channel-0 is open..."
while ! secretcli query ibc channel end channel-0 --output json | jq -r .state; do
    sleep 1
done
echo "IBC channel-0 is open!"