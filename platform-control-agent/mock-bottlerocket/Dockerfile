FROM rust:1.82-slim AS builder

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mock-bottlerocket /usr/local/bin/

# Create socket directory
RUN mkdir -p /run

EXPOSE 8080

CMD ["/usr/local/bin/mock-bottlerocket"]
