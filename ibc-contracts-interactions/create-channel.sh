docker exec hermes-relayer                                 \
    hermes --config /home/hermes-user/.hermes/alternative-config.toml   \
    create channel                                                      \
    --a-chain secretdev-1                                               \
    --a-port wasm.$(head -n 1 ./contract-upload/contract-addresses.log)                 \
    --b-port wasm.$(tail -n 1 ./contract-upload/contract-addresses.log)                 \
    --a-connection connection-0
