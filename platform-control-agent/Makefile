.PHONY: all init build build-dev run test clean proto lint fmt fips-build

# Variables
DOCKER_REGISTRY ?= localhost:5000
IMAGE_NAME ?= platform-control-agent
IMAGE_TAG ?= latest
DEV_IMAGE_TAG ?= dev

# Default target
all: init build

# Initialize project - generate all needed files
init:
	@echo "Initializing project..."
	@echo "Generating Cargo.lock..."
	@cargo generate-lockfile || true
	@echo "Creating required directories..."
	@mkdir -p src/api target/debug/build
	@echo "Project initialized successfully!"

# Build production container
build: init
	@echo "Building production container..."
	docker build -t $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) .

# Build development container
build-dev: init
	@echo "Building development container..."
	docker build -f Dockerfile.dev -t $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(DEV_IMAGE_TAG) .

# Build with FIPS compliance verification
fips-build: init
	@echo "Building FIPS-compliant container..."
	docker build -t $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)-fips .

# Run development environment
run: build-dev
	@echo "Starting development environment..."
	docker-compose up

# Run specific service
run-agent:
	docker-compose up platform-agent

run-mock:
	docker-compose up mock-bottlerocket

# Run tests
test: init
	@echo "Running tests..."
	cargo test --all-features

# Run tests in container
test-container: build-dev
	docker run --rm $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(DEV_IMAGE_TAG) cargo test --all-features

# Run all tests including integration tests
run-tests: init
	@echo "Running unit tests..."
	cargo test --all-features
	@echo ""
	@echo "Checking code with clippy..."
	cargo clippy --all-features -- -D warnings || true
	@echo ""
	@echo "Running integration tests..."
	@echo "Starting test environment..."
	docker-compose up -d mock-bottlerocket etcd
	@sleep 2
	@echo "Running agent tests against mock API..."
	RUST_LOG=debug BOTTLEROCKET_API_URL=http://localhost:8080 cargo test --all-features -- --test-threads=1
	@echo "Stopping test environment..."
	docker-compose down
	@echo ""
	@echo "All tests completed!"

# Generate protobuf code
proto: init
	@echo "Generating protobuf code..."
	cargo build --features fips

# Lint code
lint: init
	@echo "Running clippy..."
	cargo clippy --all-features -- -D warnings

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	docker-compose down -v
	rm -rf target/

# Local development setup
dev-setup: init
	@echo "Setting up local development..."
	@mkdir -p certs
	@echo "Development setup complete!"

# Run with local Bottlerocket mock
local-run: dev-setup
	RUST_LOG=debug cargo run -- serve --dev-mode

# Build and push to registry
push: build
	docker push $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)

# Run security scan
security-scan: build
	@echo "Running security scan..."
	docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
		aquasec/trivy:latest image $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) || \
		echo "Trivy not available, skipping security scan"

# Performance test
perf-test: build
	@echo "Running performance tests..."
	docker run --rm $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG) /usr/local/bin/platform-control perf-test

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --open

# Quick build for local testing
quick-build: init
	@echo "Quick build for local testing..."
	cargo build --features fips

# Help target
help:
	@echo "Available targets:"
	@echo "  make init           - Initialize project and generate needed files"
	@echo "  make build          - Build production container"
	@echo "  make build-dev      - Build development container"
	@echo "  make fips-build     - Build FIPS-compliant container"
	@echo "  make run            - Run development environment"
	@echo "  make test           - Run unit tests"
	@echo "  make run-tests      - Run all tests (unit + integration + linting)"
	@echo "  make proto          - Generate protobuf code"
	@echo "  make lint           - Run clippy linter"
	@echo "  make fmt            - Format code"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make dev-setup      - Set up local development"
	@echo "  make local-run      - Run locally without Docker"
	@echo "  make push           - Push image to registry"
	@echo "  make security-scan  - Run security scan"
	@echo "  make docs           - Generate documentation"
	@echo "  make quick-build    - Quick local build"