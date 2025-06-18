#!/bin/bash

# Demo script to show MCP client-proxy-server communication flow
# This script demonstrates how data flows through the proxy

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

clear

echo -e "${CYAN}=== MCP Proxy TUI Demo ===${NC}"
echo
echo "This demo shows how MCP messages flow through the proxy:"
echo
echo -e "${GREEN}[MCP Client]${NC} ---> ${BLUE}[MCP Proxy]${NC} ---> ${YELLOW}[MCP Server]${NC}"
echo -e "     |                   |                   |"
echo -e "     |                   v                   |"
echo -e "     |            ${BLUE}[MCP Monitor]${NC}             |"
echo -e "     |                                       |"
echo -e "     <----------- responses ---------------->"
echo
echo "The monitor displays all traffic flowing through the proxy in real-time."
echo

echo -e "${CYAN}Step 1: Build the project${NC}"
if [ ! -f "./target/release/mcp-proxy" ] || [ ! -f "./target/release/mcp-monitor" ]; then
    echo "Building project..."
    cargo build --release
fi
echo -e "${GREEN}✓ Binaries ready${NC}"
echo

echo -e "${CYAN}Step 2: Instructions${NC}"
echo "You'll need 2 terminals for this demo:"
echo
echo -e "Terminal 1 (Monitor): ${YELLOW}./run_test.sh monitor${NC}"
echo "  - Shows all proxy traffic in real-time"
echo "  - Displays connection status and statistics"
echo "  - Use 'q' to quit, 'c' to clear logs"
echo
echo -e "Terminal 2 (Test): ${YELLOW}./run_test.sh proxy${NC}"
echo "  - Runs a test client sending requests through proxy to server"
echo "  - Watch the monitor to see all messages"
echo

echo -e "${CYAN}What you'll see in the monitor:${NC}"
echo "- Proxy connection events"
echo "- Request messages (→) from client to server"
echo "- Response messages (←) from server to client"
echo "- Timing information and statistics"
echo "- Any errors or warnings"
echo

echo -e "${CYAN}Example message flow:${NC}"
echo -e "${GREEN}→ Client:${NC} {\"method\":\"initialize\",\"id\":1}"
echo -e "${BLUE}  Proxy logs this request${NC}"
echo -e "${YELLOW}← Server:${NC} {\"result\":{\"protocolVersion\":\"2024-11-05\"}}"
echo -e "${BLUE}  Proxy logs this response${NC}"
echo

echo -e "${CYAN}Ready to start!${NC}"
echo "Open another terminal and run: ./run_test.sh monitor"
echo "Then come back here and press Enter to run the proxy test..."
read

./run_test.sh proxy