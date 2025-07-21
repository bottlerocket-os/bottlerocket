# Platform Control Agent

API-driven control agent for Bottlerocket nodes, providing Talos-like management capabilities with FIPS compliance.

## Overview

The Platform Control Agent runs as a privileged host container on Bottlerocket nodes, exposing a gRPC API for:
- Declarative machine configuration
- Cluster bootstrapping
- Update orchestration
- Compliance enforcement

## Features

- **API-Driven Management**: Full machine lifecycle management via gRPC API
- **FIPS 140-3 Compliance**: Built with FIPS-compliant cryptography for FedRAMP environments
- **Kubernetes Integration**: Seamless cluster bootstrap and management
- **Multi-Infrastructure**: Support for vSphere, bare metal, and cloud providers
- **Secure by Default**: mTLS authentication and encrypted communications
- **Prometheus Metrics**: Built-in observability and monitoring
- **Development Mode**: Easy local testing with mock services

## Quick Start (macOS Development)

### Prerequisites

- Docker Desktop for Mac
- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Make
- protoc (Protocol Buffers compiler)
- OpenSSL (for certificate generation)
- grpcurl (optional, for API testing)

### Initial Setup

```bash
# Initialize the project
make init

# Generate development certificates
./scripts/generate-dev-certs.sh

# Create environment file (optional - defaults provided)
cp .env.example .env
# Edit .env to customize ports, paths, and other settings
```

### Building

```bash
# Build development container
make build-dev

# Build production container
make build

# Build FIPS-compliant container
make fips-build
```

### Running Locally

```bash
# Start the complete development stack
make run

# Or run individual components
make run-agent    # Platform agent only
make run-mock     # Mock Bottlerocket API only

# Run locally without Docker
make local-run
```

### Testing

```bash
# Run all tests (unit + integration + linting)
make run-tests

# Run unit tests only
make test

# Run tests in container
make test-container

# Lint code
make lint

# Format code
make fmt

# Security scan
make security-scan
```

## Architecture

```
Platform Control Agent
├── gRPC API Server (port 50000)
│   ├── Machine configuration
│   ├── Status reporting
│   └── Event streaming
├── Bottlerocket Client
│   ├── Unix socket support (production)
│   ├── HTTP support (development)
│   └── Settings API integration
└── Service Implementation
    ├── Configuration translator
    ├── Validation engine
    └── State management
```

### Transport Support

The Platform Control Agent supports both Unix domain sockets and HTTP for connecting to the Bottlerocket API:

- **Production Mode**: Connects via Unix socket at `unix:///run/api.sock`
- **Development Mode**: Connects via HTTP to mock API or remote endpoints
- **Auto-detection**: Protocol is determined by URL scheme (`unix://` vs `http://`)

The client uses `hyperlocal` for Unix socket support, providing seamless HTTP-over-Unix-socket communication.

## Development Stack

The `docker-compose.yml` provides a complete development environment:

- **platform-agent**: The main control agent
- **mock-bottlerocket**: Mock Bottlerocket API for testing
- **etcd**: For testing cluster operations
- **prometheus**: Metrics collection
- **grafana**: Metrics visualization
- **grpcui**: Web UI for testing gRPC endpoints

Access points:
- gRPC API: `localhost:50000`
- Mock Bottlerocket API: `localhost:8080`
- Prometheus: `localhost:9091`
- Grafana: `localhost:3000` (admin/admin)
- gRPC UI: `localhost:8081`

## API Usage

### Using grpcurl

List available services:
```bash
grpcurl -plaintext localhost:50000 list
```

### Apply Configuration

```bash
# Using grpcurl
grpcurl -plaintext -d '{
  "config": {
    "version": "1.0.0",
    "type": "MACHINE_TYPE_CONTROL_PLANE",
    "cluster": {
      "name": "my-cluster",
      "endpoint": "https://api.cluster.local:6443",
      "ca_certificate": "LS0tLS1CRUdJTi...",
      "dns_ip": "10.96.0.10"
    }
  }
}' localhost:50000 platform.machine.v1alpha1.MachineService/ApplyConfiguration
```

### Get Status

```bash
grpcurl -plaintext localhost:50000 platform.machine.v1alpha1.MachineService/GetStatus
```

### With TLS (Production)

```bash
# List services with TLS
grpcurl -cacert certs/ca.crt \
  -cert certs/client.crt \
  -key certs/client.key \
  localhost:50000 list

# Apply configuration with TLS
grpcurl -cacert certs/ca.crt \
  -cert certs/client.crt \
  -key certs/client.key \
  -d '{"config": {...}}' \
  localhost:50000 platform.machine.v1alpha1.MachineService/ApplyConfiguration
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PLATFORM_AGENT_PORT` | gRPC server port | `50000` |
| `PLATFORM_AGENT_METRICS_PORT` | Metrics port | `9090` |
| `BOTTLEROCKET_API_URL` | Bottlerocket API endpoint | `unix:///run/api.sock` |
| `TLS_ENABLED` | Enable TLS | `true` |
| `TLS_CERT_PATH` | Server certificate path | `/app/certs/server.crt` |
| `TLS_KEY_PATH` | Server key path | `/app/certs/server.key` |
| `TLS_CA_PATH` | CA certificate path | `/app/certs/ca.crt` |
| `RUST_LOG` | Log level | `info` |
| `DEV_MODE` | Development mode (disables TLS) | `false` |
| `OPENSSL_FIPS` | Enable FIPS mode | `1` (in FIPS builds) |

### Development vs Production

Development mode (`DEV_MODE=true`):
- TLS disabled for easier testing
- Mock Bottlerocket API available
- Debug logging enabled
- Prometheus and Grafana included

Production mode:
- TLS required with client certificates
- Connects to real Bottlerocket API via Unix socket
- Structured JSON logging
- External monitoring integration

## FIPS Compliance

The agent is built with FIPS-compliant cryptographic libraries:
- OpenSSL with FIPS module
- FIPS-approved TLS cipher suites
- Validated crypto throughout the stack

## Production Deployment

1. Build the FIPS-compliant container:
   ```bash
   make fips-build
   ```

2. Push to your registry:
   ```bash
   DOCKER_REGISTRY=your-registry.io make push
   ```

3. Configure Bottlerocket user-data:
   ```toml
   [settings.host-containers.platform-control]
   enabled = true
   source = "your-registry.io/platform-control-agent:latest-fips"
   superpowered = true
   ```

## Project Structure

```
platform-control-agent/
├── src/
│   ├── api/               # Protobuf definitions
│   │   └── machine.proto  # Machine service API
│   ├── services/          # gRPC service implementations
│   │   └── machine_service.rs
│   ├── bottlerocket/      # Bottlerocket API client
│   │   ├── client.rs      # HTTP client implementation
│   │   └── mod.rs
│   └── main.rs           # Application entry point
├── monitoring/            # Prometheus/Grafana configs
│   ├── prometheus.yml     # Prometheus scrape config
│   └── grafana/          # Grafana datasources
├── mock-bottlerocket/     # Mock API for testing
│   └── Dockerfile        # Mock server container
├── scripts/              # Utility scripts
│   └── generate-dev-certs.sh  # TLS certificate generation
├── certs/               # Generated certificates (git-ignored)
├── target/              # Build artifacts (git-ignored)
├── Cargo.toml           # Rust dependencies
├── Cargo.lock           # Dependency lock file
├── build.rs             # Protobuf code generation
├── Dockerfile           # Production container
├── Dockerfile.dev       # Development container
├── docker-compose.yml   # Development stack
├── .env                 # Environment variables (git-ignored)
└── Makefile            # Build automation
```

## Monitoring

### Prometheus Metrics

The agent exposes metrics on the configured metrics port (default: 9090):
- `platform_requests_total` - Total API requests by method
- `platform_request_duration_seconds` - Request latency histogram
- `platform_errors_total` - Error count by type
- `platform_bottlerocket_api_calls_total` - Bottlerocket API call metrics
- `platform_bottlerocket_api_errors_total` - Bottlerocket API errors

Access metrics: `curl http://localhost:9090/metrics`

### Grafana Dashboards

Access Grafana at http://localhost:3000 (admin/admin) for:
- API request rates and latencies
- Error tracking and alerting
- System health metrics
- Bottlerocket API performance

## Troubleshooting

### Common Issues

#### Port Conflicts
```bash
# Error: bind: address already in use
# Solution: Change ports in docker-compose.yml or .env file
# Example: Change etcd from 2379 to 12379
```

#### Build Issues
```bash
# Missing Cargo.lock
make init

# Protobuf compilation errors
# Ensure protoc is installed:
brew install protobuf  # macOS
apt-get install protobuf-compiler  # Linux

# FIPS build failures
# Check OpenSSL version and ensure OPENSSL_FIPS=1 is set
```

#### Connection Issues
```bash
# Check services are running
docker-compose ps

# View logs
docker-compose logs platform-agent
docker-compose logs mock-bottlerocket

# Test mock API
curl http://localhost:8080/os

# Test agent health
grpcurl -plaintext localhost:50000 list
```

#### Certificate Issues
```bash
# Regenerate certificates
./scripts/generate-dev-certs.sh

# Verify certificate paths
ls -la certs/
```

### Debug Commands

```bash
# Enable debug logging
RUST_LOG=debug make local-run

# Test with mock API
BOTTLEROCKET_API_URL=http://localhost:8080 cargo run -- serve --dev-mode

# Run specific test
cargo test test_settings_serialization

# Check for port usage
lsof -i :50000  # macOS
netstat -tuln | grep 50000  # Linux
```

## Security Considerations

- **Always use TLS in production** - Never run with `DEV_MODE=true` in production
- **Rotate certificates regularly** - Implement automated certificate rotation
- **Limit network exposure** - Use firewall rules to restrict API access
- **Use strong authentication** - Implement mTLS with client certificates
- **Follow Bottlerocket security best practices** - Run as host container with minimal privileges
- **Enable audit logging** - Track all configuration changes
- **Use FIPS builds for compliance** - Required for FedRAMP environments

## Performance Tuning

- **Connection pooling**: The agent maintains a connection pool to Bottlerocket API
- **Request timeouts**: Default 30s, configurable via environment
- **Concurrent requests**: Supports multiple concurrent gRPC streams
- **Metrics cardinality**: Be careful with label cardinality in Prometheus

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Run the test suite: `make run-tests`
6. Format code: `make fmt`
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

### Code Style

- Follow Rust idioms and conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Add documentation for public APIs
- Write unit tests for new functionality

## License

MIT OR Apache-2.0