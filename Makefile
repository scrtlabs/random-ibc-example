all:
	cd demo/consumer && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-tester:v1.0.0 .
	cd demo/proxy && docker build -f Dockerfile -t ghcr.io/scrtlabs/random-proxy-contract-uploader:v1.0.0 .