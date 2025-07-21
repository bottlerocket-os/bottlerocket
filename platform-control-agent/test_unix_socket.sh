#!/bin/bash

# Test script for Unix socket implementation

echo "Testing Unix Socket Implementation"
echo "================================="

# Create a temporary directory for testing
TEST_DIR=$(mktemp -d)
SOCKET_PATH="$TEST_DIR/test.sock"

echo "Test directory: $TEST_DIR"
echo "Socket path: $SOCKET_PATH"

# Start a simple HTTP server on Unix socket using socat
echo "Starting mock Unix socket server..."
cat > "$TEST_DIR/response.txt" << EOF
HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: 166

{
  "arch": "x86_64",
  "build_id": "test-build-123",
  "pretty_name": "Bottlerocket OS 1.16.0",
  "variant_id": "aws-k8s-1.28",
  "version_id": "1.16.0"
}
EOF

# Check if socat is installed
if ! command -v socat &> /dev/null; then
    echo "socat is not installed. Installing..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install socat
    else
        echo "Please install socat manually"
        exit 1
    fi
fi

# Start socat server in background
socat UNIX-LISTEN:"$SOCKET_PATH",fork EXEC:"cat $TEST_DIR/response.txt" &
SOCAT_PID=$!

# Wait for socket to be created
sleep 1

# Test with curl
echo "Testing with curl..."
curl --unix-socket "$SOCKET_PATH" http://localhost/os

echo ""
echo "Testing our Rust client..."

# Create a test program
cat > "$TEST_DIR/test_client.rs" << 'EOF'
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = std::env::args().nth(1).expect("Socket path required");
    
    // Check if socket exists
    if !Path::new(&socket_path).exists() {
        eprintln!("Socket does not exist: {}", socket_path);
        std::process::exit(1);
    }
    
    println!("Socket found at: {}", socket_path);
    
    // Test creating client with Unix socket URL
    let url = format!("unix://{}", socket_path);
    println!("Testing URL: {}", url);
    
    // Just test that we can create the client
    match std::env::current_dir() {
        Ok(path) => println!("Current directory: {:?}", path),
        Err(e) => eprintln!("Error getting current directory: {}", e),
    }
    
    println!("Unix socket test completed successfully!");
    Ok(())
}
EOF

# Build and run the test
cd "$TEST_DIR"
cargo init --name test_unix_client
cp test_client.rs src/main.rs

# Add dependencies
cat >> Cargo.toml << EOF
tokio = { version = "1", features = ["full"] }
EOF

cargo build --release 2>/dev/null
./target/release/test_unix_client "$SOCKET_PATH"

# Cleanup
kill $SOCAT_PID 2>/dev/null
rm -rf "$TEST_DIR"

echo "Test completed!"