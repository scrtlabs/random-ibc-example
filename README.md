## Secret IBC setup
Two local secrets can Inter-blockchainly communicate with each other via a Hermes relayer

### Build (Run from hermes-container)
```bash
docker build -f hermes.Dockerfile . --tag hermes:test
```

### Run (Run from the repository's root directory)
```bash
docker compose up
```

### Verify IBC transfers
Assuming you have a key 'a' which is not the relayer's key,
from localhost:
```bash
# be on the source network (secretdev-1)
secretcli config node http://localhost:26657

# check the initial balance of a
secretcli q bank balances $(secretcli keys list | jq -r '.[] | select(.name=="a") | .address') | jq

# transfer to the destination network (Wait long enough for the channel to be created in the hermes-relayer container)
secretcli tx ibc-transfer transfer transfer channel-0 secret1fc3fzy78ttp0lwuujw7e52rhspxn8uj52zfyne 2uscrt --from a

# check a's balance after transfer
secretcli q bank balances $(secretcli keys list | jq -r '.[] | select(.name=="a") | .address') | jq

# switch to the destination network (secretdev-2)
secretcli config node http://localhost:36657

# check that you have an ibc-denom
secretcli q bank balances secret1fc3fzy78ttp0lwuujw7e52rhspxn8uj52zfyne | jq # should have 2 ibc denom
```

### Verify IBC Between contracts
the `ibc-contracts-interactions` folder has further instructions