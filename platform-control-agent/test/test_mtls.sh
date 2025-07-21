#!/bin/bash

# Test mTLS functionality

echo "=== mTLS Test ==="
echo

# Generate test certificates
echo "1. Generating test certificates..."
mkdir -p test/certs

# Generate CA
openssl req -x509 -newkey rsa:4096 -keyout test/certs/ca-key.pem -out test/certs/ca-cert.pem -days 365 -nodes -subj "/CN=Test CA"

# Generate server cert
openssl req -newkey rsa:4096 -keyout test/certs/server-key.pem -out test/certs/server-req.pem -nodes -subj "/CN=localhost"
openssl x509 -req -in test/certs/server-req.pem -CA test/certs/ca-cert.pem -CAkey test/certs/ca-key.pem -CAcreateserial -out test/certs/server-cert.pem -days 365

# Generate client cert
openssl req -newkey rsa:4096 -keyout test/certs/client-key.pem -out test/certs/client-req.pem -nodes -subj "/CN=test-client"
openssl x509 -req -in test/certs/client-req.pem -CA test/certs/ca-cert.pem -CAkey test/certs/ca-key.pem -CAcreateserial -out test/certs/client-cert.pem -days 365

echo "✓ Certificates generated"
echo

# Test 2: Start server with mTLS (in background)
echo "2. Starting server with mTLS..."
echo "Run this command in another terminal:"
echo "cargo run -- serve -b 0.0.0.0:50052 --tls-cert test/certs/server-cert.pem --tls-key test/certs/server-key.pem --tls-ca test/certs/ca-cert.pem"
echo

# Test 3: Test with grpcurl using client cert
echo "3. Testing with client certificate..."
echo "Command to test mTLS connection:"
echo "grpcurl -cert test/certs/client-cert.pem -key test/certs/client-key.pem -cacert test/certs/ca-cert.pem localhost:50052 platform.machine.v1alpha1.MachineService/GetStatus"
echo

echo "4. Testing without client certificate (should fail)..."
echo "Command to test without client cert:"
echo "grpcurl -cacert test/certs/ca-cert.pem localhost:50052 platform.machine.v1alpha1.MachineService/GetStatus"
echo

echo "mTLS test setup complete!"