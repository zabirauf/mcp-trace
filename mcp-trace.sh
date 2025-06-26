#!/bin/bash

# Convenience script to run mcp-trace from the project directory
# This script will build and run the mcp-trace binary

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "$SCRIPT_DIR"

# Build the project if needed
if [ ! -f "target/release/mcp-trace" ] || [ src -nt "target/release/mcp-trace" ]; then
    echo "Building mcp-trace..."
    cargo build --release --bin mcp-trace
fi

# Run mcp-probe with all arguments passed through
exec target/release/mcp-trace "$@"