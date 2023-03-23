#!/bin/bash

# Set the path to the file containing the mnemonics
mnemonic_file="a.mnemonic"

secretcli config chain-id secretdev-1
secretcli config keyring-backend test
secretcli config output json

# Import the mnemonics into secretcli
secretcli keys add mykey --recover <<< "$(cat "$mnemonic_file")"