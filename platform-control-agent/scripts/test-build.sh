#!/bin/bash
set -euo pipefail

echo "Testing Platform Control Agent build..."

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        exit 1
    fi
}

# Check Docker is running
docker info > /dev/null 2>&1
print_status $? "Docker is running"

# Build mock Bottlerocket API
echo "Building mock Bottlerocket API..."
cd mock-bottlerocket
docker build -t mock-bottlerocket:test .
print_status $? "Mock Bottlerocket API built"
cd ..

# Build development container
echo "Building development container..."
docker build -f Dockerfile.dev -t platform-control:dev .
print_status $? "Development container built"

# Build production container
echo "Building production container..."
docker build -t platform-control:latest .
print_status $? "Production container built"

# Test container runs
echo "Testing container startup..."
docker run --rm platform-control:latest help > /dev/null 2>&1
print_status $? "Container runs successfully"

echo -e "\n${GREEN}All tests passed!${NC}"
echo "You can now run 'make run' to start the development environment."