#!/bin/bash

# Run test script for MCP Proxy TUI
# Usage: 
#   ./run_test.sh proxy   - Runs a test MCP client through proxy to test server
#   ./run_test.sh monitor - Runs the monitor to view proxy traffic

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if binaries exist
check_binaries() {
    if [ ! -f "./target/release/mcp-proxy" ]; then
        print_error "mcp-proxy binary not found. Run 'cargo build --release' first."
        exit 1
    fi
    
    if [ ! -f "./target/release/mcp-monitor" ]; then
        print_error "mcp-monitor binary not found. Run 'cargo build --release' first."
        exit 1
    fi
    
    if [ ! -f "./test-mcp-server/test_server.py" ]; then
        print_error "test_server.py not found in test-mcp-server directory."
        exit 1
    fi
    
    if [ ! -f "./test-mcp-server/test_client.py" ]; then
        print_error "test_client.py not found in test-mcp-server directory."
        exit 1
    fi
}

# Function to run the proxy test
run_proxy_test() {
    print_info "Starting MCP Proxy Test Setup"
    echo
    print_info "This will run: Client -> Proxy -> Server"
    print_info "The proxy will connect to the monitor if it's running"
    echo
    
    # Generate a unique proxy name
    PROXY_NAME="test-proxy-$(date +%s)"
    
    print_info "Starting test with proxy name: $PROXY_NAME"
    echo
    
    # Command to run the test server through the proxy
    PROXY_CMD="./target/release/mcp-proxy --name \"$PROXY_NAME\" --command \"python3 test-mcp-server/test_server.py --fast\" --verbose"
    
    print_info "Running command:"
    echo "  Client: python3 test-mcp-server/test_client.py"
    echo "  Proxy:  $PROXY_CMD"
    echo "  Server: python3 test-mcp-server/test_server.py --fast"
    echo
    
    print_warning "Make sure to run './run_test.sh monitor' in another terminal to see the traffic!"
    echo
    print_info "Starting in 3 seconds..."
    sleep 3
    
    # Run the client piped through the proxy
    python3 test-mcp-server/test_client.py 2>&1 | $PROXY_CMD
    
    print_success "Test completed!"
}

# Function to run the monitor
run_monitor() {
    print_info "Starting MCP Monitor"
    echo
    print_info "The monitor will display all proxy traffic in real-time"
    print_info "Press 'q' to quit, 'c' to clear logs, '↑/↓' to scroll"
    echo
    
    # Clean up any existing socket
    rm -f /tmp/mcp-monitor.sock
    
    # Run the monitor
    ./target/release/mcp-monitor --verbose
}

# Function to run a demo with both client and server through separate proxies
run_demo() {
    print_info "Starting Full Demo Setup"
    echo
    print_info "This demo shows bidirectional communication through proxies"
    echo
    
    print_warning "This requires 3 terminals:"
    echo "  1. This terminal (runs the demo)"
    echo "  2. Another terminal for: ./run_test.sh monitor"
    echo "  3. Optional: tail -f logs to see detailed output"
    echo
    
    print_info "Press Enter to continue..."
    read
    
    # Start the test server with a proxy
    print_info "Starting test server with proxy..."
    ./target/release/mcp-proxy \
        --name "server-proxy" \
        --command "python3 test-mcp-server/test_server.py --fast" \
        --verbose 2>server-proxy.log &
    SERVER_PROXY_PID=$!
    
    sleep 2
    
    # Run the client through another proxy
    print_info "Starting test client with proxy..."
    python3 test-mcp-server/test_client.py 2>&1 | \
        ./target/release/mcp-proxy \
        --name "client-proxy" \
        --command "cat" \
        --verbose 2>client-proxy.log
    
    # Clean up
    print_info "Cleaning up..."
    kill $SERVER_PROXY_PID 2>/dev/null || true
    
    print_success "Demo completed!"
    echo
    print_info "Check server-proxy.log and client-proxy.log for detailed output"
}

# Main script logic
case "$1" in
    proxy)
        check_binaries
        run_proxy_test
        ;;
    monitor)
        check_binaries
        run_monitor
        ;;
    demo)
        check_binaries
        run_demo
        ;;
    simple)
        check_binaries
        print_info "Running simple demo: Client -> Proxy -> Server"
        echo
        print_warning "Run './run_test.sh monitor' in another terminal first!"
        echo
        sleep 2
        python3 test-mcp-server/simple_demo.py 2>&1 | \
            ./target/release/mcp-proxy \
                --name "simple-demo" \
                --command "python3 test-mcp-server/test_server.py --fast" \
                --verbose
        ;;
    *)
        echo "Usage: $0 {proxy|monitor|demo|simple}"
        echo
        echo "Commands:"
        echo "  proxy   - Run a test MCP client through proxy to test server"
        echo "  monitor - Run the monitor to view proxy traffic"
        echo "  demo    - Run a full demo with multiple proxies"
        echo "  simple  - Run a simple demo with basic MCP requests"
        echo
        echo "Example workflow:"
        echo "  Terminal 1: ./run_test.sh monitor"
        echo "  Terminal 2: ./run_test.sh proxy"
        echo
        echo "For a quick demo:"
        echo "  Terminal 1: ./run_test.sh monitor"
        echo "  Terminal 2: ./run_test.sh simple"
        exit 1
        ;;
esac