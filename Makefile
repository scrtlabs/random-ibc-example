build: build-contracts build-images
build-contracts: demo/proxy/ibc_random_generator.wasm demo/proxy/ibc_proxy.wasm demo/consumer/consumer.wasm

build-images:
	cd demo/consumer && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-tester:v1.0.0 .
	cd demo/proxy && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-contract-uploader:v1.0.0 .

demo/proxy/ibc_random_generator.wasm: contracts/generator/src/*.rs contracts/generator/Cargo.toml
	cd contracts/generator && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp contracts/generator/target/wasm32-unknown-unknown/release/ibc_random_generator.wasm demo/proxy/ibc_random_generator.wasm

demo/proxy/ibc_proxy.wasm: contracts/proxy/src/*.rs contracts/proxy/Cargo.toml
	cd contracts/proxy && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp contracts/proxy/target/wasm32-unknown-unknown/release/ibc_proxy.wasm demo/proxy/ibc_proxy.wasm

demo/consumer/consumer.wasm: contracts/consumer/src/*.rs contracts/consumer/Cargo.toml
	cd contracts/consumer && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp contracts/consumer/target/wasm32-unknown-unknown/release/consumer.wasm demo/consumer/consumer.wasm

.PHONY: run
run:
	docker compose up

.PHONY: clean
clean:
	rm demo/consumer/consumer.wasm
	rm demo/proxy/ibc_proxy.wasm
	rm demo/proxy/ibc_random_generator.wasm