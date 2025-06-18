#!/usr/bin/env python3
"""
Simple demo showing MCP client-server communication
This can be run through the proxy to see the message flow
"""

import json
import sys
import time

def send_request(method, params=None, request_id=1):
    """Send a JSON-RPC request"""
    request = {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method
    }
    if params:
        request["params"] = params
    
    print(json.dumps(request), flush=True)
    print(f"[Client] Sent: {method}", file=sys.stderr)

def main():
    print("[Client] Starting simple MCP demo...", file=sys.stderr)
    
    # Wait a bit for server to start
    time.sleep(0.5)
    
    # 1. Initialize connection
    send_request("initialize", {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "simple-demo", "version": "1.0"}
    }, 1)
    
    time.sleep(1)
    
    # 2. List available tools
    send_request("tools/list", None, 2)
    
    time.sleep(1)
    
    # 3. Call a tool
    send_request("tools/call", {
        "name": "calculator",
        "arguments": {"input": "2 + 2"}
    }, 3)
    
    time.sleep(1)
    
    # 4. Read a resource
    send_request("resources/read", {
        "uri": "config://settings.json"
    }, 4)
    
    time.sleep(1)
    
    print("[Client] Demo completed!", file=sys.stderr)

if __name__ == "__main__":
    main()