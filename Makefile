build-images:
	cd demo/consumer && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-tester:v1.0.0 .
	cd demo/proxy && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-contract-uploader:v1.0.0 .

build-proxy-contract:
	cd contracts/proxy && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp contracts/proxy/target/wasm32-unknown-unknown/release/ibc.wasm demo/proxy/ibc.wasm

build-consumer-contract:
	cd contracts/consumer && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp contracts/consumer/target/wasm32-unknown-unknown/release/consumer.wasm demo/consumer/consumer.wasm

build: build-proxy-contract build-consumer-contract