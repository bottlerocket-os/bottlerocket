#!/bin/bash
set -euo pipefail

# Script to generate development certificates for mTLS testing

CERT_DIR="certs"
DAYS_VALID=365

echo "Generating development certificates..."

# Create certificate directory
mkdir -p "$CERT_DIR"

# Generate CA private key
openssl genrsa -out "$CERT_DIR/ca.key" 4096

# Generate CA certificate
openssl req -new -x509 -days $DAYS_VALID -key "$CERT_DIR/ca.key" -out "$CERT_DIR/ca.crt" \
    -subj "/C=US/ST=Development/L=Local/O=Platform Control/CN=Platform CA"

# Generate server private key
openssl genrsa -out "$CERT_DIR/server.key" 4096

# Generate server certificate request
openssl req -new -key "$CERT_DIR/server.key" -out "$CERT_DIR/server.csr" \
    -subj "/C=US/ST=Development/L=Local/O=Platform Control/CN=localhost"

# Create extensions file for SAN
cat > "$CERT_DIR/server_ext.cnf" <<EOF
subjectAltName = DNS:localhost,DNS:platform-agent,DNS:*.platform.local,IP:127.0.0.1,IP:0.0.0.0
EOF

# Generate server certificate signed by CA
openssl x509 -req -days $DAYS_VALID -in "$CERT_DIR/server.csr" \
    -CA "$CERT_DIR/ca.crt" -CAkey "$CERT_DIR/ca.key" -CAcreateserial \
    -out "$CERT_DIR/server.crt" -extfile "$CERT_DIR/server_ext.cnf"

# Generate client private key
openssl genrsa -out "$CERT_DIR/client.key" 4096

# Generate client certificate request
openssl req -new -key "$CERT_DIR/client.key" -out "$CERT_DIR/client.csr" \
    -subj "/C=US/ST=Development/L=Local/O=Platform Control/CN=platform-client"

# Generate client certificate signed by CA
openssl x509 -req -days $DAYS_VALID -in "$CERT_DIR/client.csr" \
    -CA "$CERT_DIR/ca.crt" -CAkey "$CERT_DIR/ca.key" -CAcreateserial \
    -out "$CERT_DIR/client.crt"

# Clean up temporary files
rm -f "$CERT_DIR"/*.csr "$CERT_DIR"/*.srl "$CERT_DIR/server_ext.cnf"

# Set appropriate permissions
chmod 600 "$CERT_DIR"/*.key
chmod 644 "$CERT_DIR"/*.crt

echo "Certificates generated successfully in $CERT_DIR/"
echo ""
echo "Files created:"
echo "  - ca.crt: Certificate Authority certificate"
echo "  - ca.key: Certificate Authority private key"
echo "  - server.crt: Server certificate"
echo "  - server.key: Server private key"
echo "  - client.crt: Client certificate"
echo "  - client.key: Client private key"
echo ""
echo "To use with grpcurl:"
echo "  grpcurl -cacert $CERT_DIR/ca.crt -cert $CERT_DIR/client.crt -key $CERT_DIR/client.key localhost:50000 list"