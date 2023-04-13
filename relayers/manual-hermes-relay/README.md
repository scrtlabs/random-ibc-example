## Secret Inter-Contract IBC setup
Two contracts, on two different local secrets, can Inter-blockchainly, directly communicate with each other via a Hermes relayer

#### Build (Run from hermes-container)
```bash
docker build -f hermes.Dockerfile . --tag hermes:test
```

### Run the Monitoring script (Run from ibc-contracts-interactions)
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

### Upload the contracts (Run from ibc-contracts-interactions/manual-hermes-relay)
In another terminal, The test script will store the contracts, instantiate them, save their address to helper files, and run a test to check correctness.
Notice that if you want any other test besides 'just setup contracts' to pass, you'll need to wait for hermes
to start. It might take a couple of minutes since 'docker compose up'
```bash
yarn test # this will also create ibc.wasm
```

### Tell the relayer to open a channel between the contracts (Run from ibc-contracts-interactions)
```bash
./create-channel.sh
```
How did the contract's queries change? (Check the terminal of ./query-contracts.sh)

### Send a packet and relay it to the other chain (Run from ibc-contracts-interactions/manual-hermes-relay)
```bash
./request-answer-for-life.sh
# Please notice that the correct X in '--src-channel channel-X' will appear in the output of ./create-channel.sh in 'ChannelId("channel-X"'
```

How did the contract's queries change?

### Finally query the contract on localsecret-1 to see the reply from localsecret-2
```bash
secretcli q compute query $(head -n 1 ../contract-addresses.log) '{"view_received_life_answer": {}}'
```
You should get the answer for life ;) (42)
