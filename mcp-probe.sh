#!/bin/bash

# Convenience script to run mcp-probe from the project directory
# This script will build and run the mcp-probe binary

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "$SCRIPT_DIR"

# Build the project if needed
if [ ! -f "target/release/mcp-probe" ] || [ src -nt "target/release/mcp-probe" ]; then
    echo "Building mcp-probe..."
    cargo build --release --bin mcp-probe
fi

# Run mcp-probe with all arguments passed through
exec target/release/mcp-probe "$@"