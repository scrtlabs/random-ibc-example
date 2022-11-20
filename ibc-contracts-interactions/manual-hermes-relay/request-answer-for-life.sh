secretcli tx compute execute $(head -n 1 ../contract-addresses.log) \
    '{"request_life_answer_from_other_chain": {"job_id": "some-job-id"}}' --from a -y

sleep 4

docker exec hermes-relayer                                 \
    hermes --config /home/hermes-user/.hermes/alternative-config.toml   \
    tx packet-recv --dst-chain secretdev-2 --src-chain secretdev-1      \
    --src-port wasm.$(head -n 1 ../contract-addresses.log)               \
    --src-channel channel-1
