#!/bin/bash
set -e

# Generate test certificates for Docker Compose bootstrap cluster
echo "Generating test certificates for bootstrap cluster..."

# Create certificates directory structure
mkdir -p certs/{ca,node1,node2,node3}

# Generate CA private key
openssl genrsa -out certs/ca/ca.key 4096

# Generate CA certificate
openssl req -new -x509 -days 365 -key certs/ca/ca.key -out certs/ca/ca.crt -subj "/C=US/ST=WA/L=Seattle/O=BottlerocketBootstrap/CN=Bootstrap-CA"

echo "Generated CA certificate"

# Function to generate node certificates
generate_node_cert() {
    local node=$1
    local hostname=$2
    
    echo "Generating certificate for $node ($hostname)"
    
    # Generate private key
    openssl genrsa -out certs/$node/tls.key 2048
    
    # Generate certificate signing request
    openssl req -new -key certs/$node/tls.key -out certs/$node/tls.csr -subj "/C=US/ST=WA/L=Seattle/O=BottlerocketBootstrap/CN=$hostname"
    
    # Create extensions file for SAN
    cat > certs/$node/tls.ext << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = $hostname
DNS.2 = localhost
IP.1 = 127.0.0.1
EOF
    
    # Generate certificate signed by CA
    openssl x509 -req -in certs/$node/tls.csr -CA certs/ca/ca.crt -CAkey certs/ca/ca.key -CAcreateserial -out certs/$node/tls.crt -days 365 -extfile certs/$node/tls.ext
    
    # Copy CA cert to node directory for trust
    cp certs/ca/ca.crt certs/$node/ca.crt
    
    # Clean up CSR and extensions file
    rm certs/$node/tls.csr certs/$node/tls.ext
    
    echo "Generated certificates for $node"
}

# Generate certificates for each node
generate_node_cert "node1" "bootstrap-node-1"
generate_node_cert "node2" "bootstrap-node-2" 
generate_node_cert "node3" "bootstrap-node-3"

echo "All certificates generated successfully!"
echo "CA certificate: certs/ca/ca.crt"
echo "Node certificates: certs/node{1,2,3}/tls.{crt,key}"

# Set proper permissions
chmod 600 certs/*/tls.key certs/ca/ca.key
chmod 644 certs/*/tls.crt certs/*/ca.crt certs/ca/ca.crt

echo "Certificate permissions set correctly"