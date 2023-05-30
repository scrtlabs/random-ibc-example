#!/usr/bin/env bash

set -ex

# config cli
secretcli config node http://localsecret-1:26657
secretcli config keyring-backend test
secretcli config output json
secretcli config chain-id secretdev-1

# add mnemonic to keyring
a_mnemonic="chair love bleak wonder skirt permit say assist aunt credit roast size obtain minute throw sand usual age smart exact enough room shadow charge"
echo $a_mnemonic | secretcli keys add a --recover

# Check if the contract is deployed
echo "Waiting for a connection with consumer chain..."
while ! secretcli q wasm list-code &> /dev/null; do
    sleep 1
done
echo "There is a connection with consumer chain!"

echo "Checking if any contract is deployed..."
list_code_output=$(secretcli q wasm list-contract-by-code 1)
while [ -z "$list_code_output" ] ; do
    sleep 1
    list_code_output=$(secretcli q wasm list-contract-by-code 1)
done
echo "A Contract is deployed!"

proxy_code_hash=$(secretcli q wasm list-code | jq -r ".[0].code_hash")
proxy_contract=$(secretcli q wasm list-contract-by-code 1 | jq -r ".[0].contract_address")

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
        secretcli tx compute instantiate 2 '{"init": {"rand_provider": {"address": "'$proxy_contract'", "code_hash": "'$proxy_code_hash'"}}}' --label test --output json --gas-prices 0.25uscrt --from a |
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
CONSUMER_CONTRACT="$CONTRACT_ADDRESS" python3 ui/app.py