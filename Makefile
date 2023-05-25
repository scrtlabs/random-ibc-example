build-proxy:
	cd contracts/proxy && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown \
	# cp target/wasm32-unknown-unknown/release

build-consumer:
	cd contracts/consumer && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

build: build-consumer build-proxy

all:
	cd demo/consumer && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-tester:v1.0.0 .
	cd demo/proxy && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-contract-uploader:v1.0.0 .