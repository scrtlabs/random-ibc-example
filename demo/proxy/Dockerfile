# Use the official Node.js LTS image as the base image
FROM node:lts

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    #### Base utilities ####
    netcat \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the TypeScript files and the package.json and tsconfig.json files
COPY . .

# Install dependencies and build the TypeScript code
RUN npm install && npm run build

# Define the entry point script
COPY entrypoint.sh /app
RUN chmod +x ./entrypoint.sh
ENTRYPOINT ["./entrypoint.sh"]