#!/usr/bin/env bash

set -ex

# Set the contract address
contract="secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg"
code_hash="1e3d516013e80cfcdd5be8838833c2627698c9a678a3c2494917a2255277763a"

# config cli
secretcli config node http://localsecret-1:26657
secretcli config keyring-backend test
secretcli config output json
secretcli config chain-id secretdev-1

# add mnemonic to keyring
a_mnemonic="chair love bleak wonder skirt permit say assist aunt credit roast size obtain minute throw sand usual age smart exact enough room shadow charge"
echo $a_mnemonic | secretcli keys add a --recover


# Check if the contract is deployed
echo "Checking if contract $contract is deployed..."
while ! secretcli q wasm contract $contract &> /dev/null; do
    sleep 1
done
echo "Contract $contract is deployed!"

# Check if IBC channel-0 is open
echo "Checking if IBC channel-0 is open..."
while [ "$(secretcli query ibc channel channels | jq -r .channels[0].state)" != "STATE_OPEN" ]; do
  sleep 1
done
echo "IBC channel-0 is open!"

# Wait for a few seconds for the chain to set up in case all the images are starting together
sleep 10

function wait_for_tx () {
    echo "Waiting for tx: $1"
    until (secretcli q tx "$1" &> /dev/null)
    do
        echo "$2"
        sleep 1
    done
}

# store wasm code on-chain so we could later instantiate it
export STORE_TX_HASH=$(
    secretcli tx compute store consumer.wasm --from a --gas 1200000 --gas-prices 0.25uscrt -y -b block |
    jq -r .txhash
)

wait_for_tx "$STORE_TX_HASH" "Waiting for store to finish on-chain..."

# test storing of wasm code (this doesn't touch sgx yet)
secretcli q tx "$STORE_TX_HASH" --output json |
    jq -e '.logs[].events[].attributes[] | select(.key == "code_id" and .value == "2")'

INIT_TX_HASH=$(
    yes |
        secretcli tx compute instantiate 2 '{"init": {"rand_provider": {"address": "'$contract'", "code_hash": "'$code_hash'"}}}' --label test --output json --gas-prices 0.25uscrt --from a |
        jq -r .txhash
)

wait_for_tx "$INIT_TX_HASH" "Waiting for instantiate to finish on-chain..."

export CONTRACT_ADDRESS=$(
    secretcli q tx "$INIT_TX_HASH" --output json |
        jq -er '.logs[].events[].attributes[] | select(.key == "contract_address") | .value' |
        head -1
)


SEND_TX_HASH=$(
    yes |
        secretcli tx compute execute --from a "$CONTRACT_ADDRESS" '{"do_something": {}}' --gas-prices 0.25uscrt --output json 2> /dev/null |
        jq -r .txhash
)

wait_for_tx "$SEND_TX_HASH" "Waiting for transfer to finish on-chain..."

secretcli q compute tx "$SEND_TX_HASH" --output json | jq

# Start the web server
python3 ui/app.py