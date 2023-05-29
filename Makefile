build-images:
	cd demo/consumer && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-tester:v1.0.0 .
	cd demo/proxy && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-contract-uploader:v1.0.0 .

build-generator-contract:
	cd contracts/generator && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp contracts/generator/target/wasm32-unknown-unknown/release/ibc_random_generator.wasm demo/proxy/ibc_random_generator.wasm

build-proxy-contract:
	cd contracts/proxy && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp contracts/proxy/target/wasm32-unknown-unknown/release/ibc_proxy.wasm demo/proxy/ibc_proxy.wasm

build-consumer-contract:
	cd contracts/consumer && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp contracts/consumer/target/wasm32-unknown-unknown/release/consumer.wasm demo/consumer/consumer.wasm

build: build-proxy-contract build-consumer-contract