FROM ubuntu:latest

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    #### Base utilities ####
    wget \
    jq \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN wget -O /usr/bin/secretcli https://github.com/scrtlabs/SecretNetwork/releases/download/v1.8.0/secretcli-Linux
RUN chmod +x /usr/bin/secretcli

# Set the working directory
WORKDIR /app

COPY send_random_request.sh .
COPY consumer.wasm .

RUN chmod +x send_random_request.sh

# Run the script when the container starts
CMD ["./send_random_request.sh"]