FROM ubuntu:latest

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    #### Base utilities ####
    wget \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN wget -O /usr/bin/secretcli https://github.com/scrtlabs/SecretNetwork/releases/download/v1.8.0/secretcli-Linux
RUN chmod +x /usr/bin/secretcli

# Set the working directory
WORKDIR /app

# Copy the script into the container
COPY configure-cli.sh .
# Make the script executable
RUN chmod +x configure-cli.sh

COPY send_random_request.sh .
RUN chmod +x send_random_request.sh


COPY ibc.wasm ibc.wasm
COPY c.mnemonic a.mnemonic

RUN ./configure-cli.sh

# Run the script when the container starts
CMD ["./send_random_request.sh"]