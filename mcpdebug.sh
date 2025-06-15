#!/bin/bash

# Convenience script to run mcpdebug from the project directory
# This script will build and run the mcpdebug binary

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
cd "$SCRIPT_DIR"

# Build the project if needed
if [ ! -f "target/release/mcpdebug" ] || [ src -nt "target/release/mcpdebug" ]; then
    echo "Building mcpdebug..."
    cargo build --release --bin mcpdebug
fi

# Run mcpdebug with all arguments passed through
exec target/release/mcpdebug "$@"