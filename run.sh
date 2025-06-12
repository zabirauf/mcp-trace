#!/bin/bash

# MCP Proxy TUI Runner Script
set -e

# Create logs directory if it doesn't exist
mkdir -p logs

# Function to show usage
show_usage() {
    echo "Usage: $0 [monitor|proxy|build|clean|logs]"
    echo ""
    echo "Commands:"
    echo "  monitor  - Run the TUI monitor (default)"
    echo "  proxy    - Run a single proxy with command"
    echo "  build    - Build Docker images"
    echo "  clean    - Clean up containers and volumes"
    echo "  logs     - Show logs from all services"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Run monitor"
    echo "  $0 monitor                           # Run monitor"
    echo "  $0 proxy python my_mcp_server.py    # Run proxy with Python server"
    echo "  $0 build                             # Build images"
    echo "  $0 clean                             # Clean up"
}

# Function to run monitor
run_monitor() {
    echo "Starting MCP Monitor..."
    docker compose up --build mcp-monitor
}

# Function to run proxy
run_proxy() {
    if [ "$#" -eq 0 ]; then
        echo "Error: No command specified for proxy"
        echo "Usage: $0 proxy <command> [args...]"
        exit 1
    fi
    
    local cmd_args=""
    for arg in "$@"; do
        cmd_args="$cmd_args \"$arg\""
    done
    
    echo "Starting MCP Proxy with command: $*"
    docker run --rm -it \
        --network tool-mcp-proxy-tui_mcp-network \
        -v "$(pwd)/logs:/app/logs" \
        -v "/tmp/mcp-sockets:/tmp/mcp-sockets" \
        -e RUST_LOG=debug \
        tool-mcp-proxy-tui_mcp-monitor \
        mcp-proxy --name "Custom Proxy" --command $cmd_args --verbose
}

# Function to build images
build_images() {
    echo "Building Docker images..."
    docker compose build
}

# Function to clean up
clean_up() {
    echo "Cleaning up containers and volumes..."
    docker compose down -v
    docker system prune -f
}

# Function to show logs
show_logs() {
    echo "Showing logs from all services..."
    docker compose logs -f
}

# Main command handling
case "${1:-monitor}" in
    monitor)
        run_monitor
        ;;
    proxy)
        shift
        run_proxy "$@"
        ;;
    build)
        build_images
        ;;
    clean)
        clean_up
        ;;
    logs)
        show_logs
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        echo "Error: Unknown command '$1'"
        echo ""
        show_usage
        exit 1
        ;;
esac