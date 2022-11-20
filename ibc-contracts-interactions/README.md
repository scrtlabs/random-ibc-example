## Secret Inter-Contract IBC setup
Two contracts, on two different local secrets, can Inter-blockchainly, directly communicate with each other via a confio relayer


### Run the Monitoring script (from ibc-contracts-interactions)
This script queries the two contracts continuously, which lets us see the state of the IBC objects within the contract,
as well as the flow of messages between the blockchains that are being relayed.
Keep this terminal in the background, and return to it after every stage in the future to see what happens.
```bash
./query-contracts.sh
```

#### If you haven't done so already, run (Run this from the repository's root directory)
```bash
docker compose up
```

#### Create the ibc.wasm file: (Run from ibc-contracts-interactions/ibc-contract)
This is the IBC contract to be uploaded on each chain
```bash
make
```

### Upload the contracts (Run from ibc-contracts-interactions/confio-relay)
Run the confio-server.ts file in order to make the confio relayer listen to WASM IBC messages.
For you convenience we put a VS-Code launch.json file, so you can open confio-server.ts and press F5
to start debug this file if you wish to inspect every stage.

### Send a packet and relay it to the other chain
```bash
secretcli tx compute execute $(head -n 1 ./contract-addresses.log) \
    '{"request_life_answer_from_other_chain": {"job_id": "some-job-id"}}' --from a -y
```
How did the contract's queries change?

### Finally query the contract on localsecret-1 to see the reply from localsecret-2
```bash
secretcli q compute query $(head -n 1 ./contract-addresses.log) '{"view_received_life_answer": {}}'
```
You should get the answer for life ;) (42)

### Notice
Currently the Hermes relayer doesn't support listening on WASM ibc messages.
Follow the README.md inside 'manual-hermes-relay' directory if you want to relay IBC messages using
the Hermes relayer, in which case you'll have to invoke hermes manually in order to relay WASM IBC
messages.
