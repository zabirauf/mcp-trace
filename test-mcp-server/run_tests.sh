#!/bin/bash

# Test script for MCP Proxy and Monitor
# This script provides easy ways to test the MCP debugging tools

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== MCP Proxy and Monitor Test Runner ==="
echo

show_help() {
    echo "Usage: $0 [OPTION]"
    echo
    echo "Options:"
    echo "  server [OPTIONS]     Start test MCP server"
    echo "  client [OPTIONS]     Start test MCP client"
    echo "  proxy [OPTIONS]      Start MCP proxy with test server"
    echo "  monitor              Start MCP monitor"
    echo "  full-test            Run complete test (monitor + proxy + client)"
    echo "  demo                 Run demo sequence"
    echo "  help                 Show this help"
    echo
    echo "Server Options:"
    echo "  --fast               Fast responses (0.1-0.5s, no errors)"
    echo "  --slow               Slow responses (2-8s, 20% errors)"
    echo "  --delay-min X        Min delay in seconds"
    echo "  --delay-max X        Max delay in seconds"
    echo "  --error-rate X       Error rate (0.0-1.0)"
    echo
    echo "Examples:"
    echo "  $0 monitor           # Start monitor in one terminal"
    echo "  $0 proxy --fast      # Start proxy with fast test server"
    echo "  $0 full-test         # Complete test setup"
    echo
}

start_server() {
    echo "Starting test MCP server..."
    python3 "$SCRIPT_DIR/test_server.py" "$@"
}

start_client() {
    echo "Starting test MCP client..."
    python3 "$SCRIPT_DIR/test_client.py" "$@"
}

start_proxy() {
    echo "Starting MCP proxy with test server..."
    cd "$PROJECT_DIR"
    
    # Build if needed
    if [ ! -f "target/release/mcp-trace" ]; then
        echo "Building mcp-trace..."
        cargo build --release
    fi
    
    # Start proxy
    ./target/release/mcp-trace proxy --command "python3 $SCRIPT_DIR/test_server.py $*" --name "TestServer"
}

start_monitor() {
    echo "Starting MCP monitor..."
    cd "$PROJECT_DIR"
    
    # Build if needed
    if [ ! -f "target/release/mcp-trace" ]; then
        echo "Building mcp-trace..."
        cargo build --release
    fi
    
    # Start monitor
    ./target/release/mcp-trace monitor
}

run_full_test() {
    echo "=== Running Full Test ==="
    echo 
    echo "This will:"
    echo "1. Start the monitor (you'll see the TUI)"
    echo "2. In another terminal, it will start a proxy with test server"
    echo "3. Then run a test client to generate traffic"
    echo
    echo "Press Enter to start monitor, then run the following in another terminal:"
    echo "  $0 proxy --fast"
    echo "  $0 client"
    echo
    read -p "Press Enter to start monitor..."
    start_monitor
}

run_demo() {
    echo "=== Demo Sequence ==="
    echo
    echo "1. Start monitor in one terminal:"
    echo "   $0 monitor"
    echo
    echo "2. Start proxy in another terminal:"
    echo "   $0 proxy --fast"
    echo  
    echo "3. Generate test traffic in a third terminal:"
    echo "   $0 client"
    echo
    echo "This will show:"
    echo "- Real-time MCP protocol messages in monitor"
    echo "- Tabbed interface (Messages, Errors, System)"
    echo "- Detailed view of requests/responses (press Enter on a log)"
    echo "- JSON formatting and word wrap controls"
    echo
}

case "$1" in
    "server")
        shift
        start_server "$@"
        ;;
    "client")
        shift
        start_client "$@"
        ;;
    "proxy")
        shift
        start_proxy "$@"
        ;;
    "monitor")
        start_monitor
        ;;
    "full-test")
        run_full_test
        ;;
    "demo")
        run_demo
        ;;
    "help"|"--help"|"-h"|"")
        show_help
        ;;
    *)
        echo "Unknown option: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac