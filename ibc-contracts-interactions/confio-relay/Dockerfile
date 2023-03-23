# Use the official Node.js LTS image as the base image
FROM node:lts

# Set the working directory
WORKDIR /app

# Copy the TypeScript files and the package.json and tsconfig.json files
COPY . .

# Install dependencies and build the TypeScript code
RUN npm install && npm run build

# Define the entry point script
COPY entrypoint.sh /
ENTRYPOINT ["/entrypoint.sh"]