# Build stage
FROM rust:1.83-bookworm AS builder

# Install dependencies for OpenSSL FIPS build
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml ./

# Generate lockfile if it doesn't exist
RUN cargo generate-lockfile

# Copy source code
COPY src ./src
COPY build.rs ./

# Build with FIPS feature enabled
ENV OPENSSL_STATIC=1
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl

# Build the application
RUN cargo build --release --features fips

# Runtime stage - minimal image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/false platform

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/platform-control /usr/local/bin/platform-control

# Create necessary directories
RUN mkdir -p /etc/platform/certs /var/lib/platform \
    && chown -R platform:platform /etc/platform /var/lib/platform

# Switch to non-root user
USER platform

# Expose gRPC port
EXPOSE 50000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/platform-control", "health"]

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/platform-control"]

# Default arguments
CMD ["serve"]