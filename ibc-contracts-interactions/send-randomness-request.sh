secretcli tx compute execute $(head -n 1 ./contract-upload/contract-addresses.log) \
    '{"request_randomness_from_other_chain": {"job_id": "job-id-that-I-set"}}' --from a -y

sleep 4

docker exec hermes-relayer                                 \
    hermes --config /home/hermes-user/.hermes/alternative-config.toml   \
    tx packet-recv --dst-chain secretdev-2 --src-chain secretdev-1      \
    --src-port wasm.$(head -n 1 ./contract-upload/contract-addresses.log)               \
    --src-channel channel-1
