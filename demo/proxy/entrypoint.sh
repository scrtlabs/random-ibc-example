#!/bin/bash

# Wait for port 26657 on host "localsecret-1" to become available
echo "Waiting for port 26657 on host 'localsecret-1'..."
while ! nc -z localsecret-1 26657; do sleep 1; done
echo "Port 26657 on host 'localsecret-1' is available!"

# Wait for port 36657 on host "localsecret-2" to become available
echo "Waiting for port 26657 on host 'localsecret-2'..."
while ! nc -z localsecret-2 26657; do sleep 1; done
echo "Port 26657 on host 'localsecret-2' is available!"

sleep 20

# Start the server
node dist/confio-server.js