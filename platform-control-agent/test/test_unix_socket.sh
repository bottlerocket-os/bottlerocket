#!/bin/bash

# Test Unix socket functionality

echo "=== Unix Socket Test ==="
echo

# Create mock Unix socket server
echo "1. Creating mock Unix socket..."
cat > /tmp/mock_unix_socket.py << 'EOF'
#!/usr/bin/env python3
import socket
import os
import json

socket_path = "/tmp/mock_api.sock"

# Remove the socket file if it already exists
if os.path.exists(socket_path):
    os.remove(socket_path)

# Create Unix socket
server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
server.bind(socket_path)
server.listen(1)

print(f"Mock Unix socket listening on {socket_path}")

while True:
    connection, client_address = server.accept()
    try:
        data = connection.recv(4096).decode()
        print(f"Received: {data[:100]}...")
        
        # Parse HTTP request
        if "GET /os" in data:
            response = {
                "arch": "x86_64",
                "build_id": "mock-build-123",
                "pretty_name": "Mock Bottlerocket OS",
                "variant_id": "aws-k8s-1.28",
                "version_id": "1.16.0-mock"
            }
            body = json.dumps(response)
        elif "PATCH /settings" in data:
            body = '{"success": true}'
        elif "POST /actions/reboot" in data:
            body = '{"success": true}'
        else:
            body = '{"error": "Not implemented"}'
        
        http_response = f"HTTP/1.1 200 OK\r\nContent-Length: {len(body)}\r\nContent-Type: application/json\r\n\r\n{body}"
        connection.sendall(http_response.encode())
    finally:
        connection.close()
EOF

chmod +x /tmp/mock_unix_socket.py
python3 /tmp/mock_unix_socket.py &
MOCK_PID=$!
sleep 2

echo "2. Testing with mock Unix socket..."
echo "Set BOTTLEROCKET_API_URL=unix:///tmp/mock_api.sock"
echo "Then restart the platform-control-agent"
echo

echo "3. Test commands:"
echo "- Get status: grpcurl -plaintext localhost:50051 platform.machine.v1alpha1.MachineService/GetStatus"
echo "- Apply config: Will use the mock Unix socket for settings API"
echo

echo "4. To stop mock server: kill $MOCK_PID"
echo
echo "Unix socket test setup complete!"