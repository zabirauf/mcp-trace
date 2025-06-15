#!/usr/bin/env python3
"""
Test MCP Client for testing the MCP Proxy and Monitor

This client sends various MCP requests to test the server and proxy functionality.
"""

import json
import sys
import time
import random
from typing import Dict, Any

class TestMCPClient:
    def __init__(self):
        self.request_id = 1
        
    def send_request(self, method: str, params: Dict[str, Any] = None) -> None:
        """Send a JSON-RPC request to stdout"""
        request = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method
        }
        
        if params:
            request["params"] = params
            
        request_json = json.dumps(request, separators=(',', ':'))
        print(request_json, flush=True)
        
        print(f"Sent: {method} (id: {self.request_id})", file=sys.stderr)
        self.request_id += 1
        
        # Small delay between requests
        time.sleep(0.1)
    
    def run_test_sequence(self):
        """Run a sequence of test requests"""
        print("Starting MCP client test sequence...", file=sys.stderr)
        
        # Initialize
        self.send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": {"listChanged": True},
                "sampling": {}
            },
            "clientInfo": {
                "name": "test-mcp-client",
                "version": "1.0.0"
            }
        })
        
        time.sleep(1)
        
        # List tools
        self.send_request("tools/list")
        time.sleep(0.5)
        
        # Call some tools
        tools = ["calculator", "web_search", "file_reader"]
        for tool in tools:
            self.send_request("tools/call", {
                "name": tool,
                "arguments": {
                    "input": f"test input for {tool}"
                }
            })
            time.sleep(random.uniform(0.5, 2.0))
        
        # List resources
        self.send_request("resources/list")
        time.sleep(0.5)
        
        # Read some resources
        resources = [
            "config://settings.json",
            "file://documents/readme.md", 
            "url://api.example.com/data"
        ]
        for resource in resources:
            self.send_request("resources/read", {"uri": resource})
            time.sleep(random.uniform(0.3, 1.5))
        
        # List prompts
        self.send_request("prompts/list")
        time.sleep(0.5)
        
        # Get prompts
        prompts = [
            {"name": "summarize", "arguments": {"text": "This is a long text that needs summarizing..."}},
            {"name": "translate", "arguments": {"text": "Hello world", "target_language": "Spanish"}}
        ]
        for prompt in prompts:
            self.send_request("prompts/get", prompt)
            time.sleep(random.uniform(0.5, 2.0))
        
        print("Test sequence completed!", file=sys.stderr)

def main():
    client = TestMCPClient()
    
    if len(sys.argv) > 1 and sys.argv[1] == "--interactive":
        print("Interactive mode - send requests manually", file=sys.stderr)
        print("Available methods: initialize, tools/list, tools/call, resources/list, resources/read, prompts/list, prompts/get", file=sys.stderr)
        while True:
            try:
                method = input("Enter method (or 'quit'): ").strip()
                if method == 'quit':
                    break
                client.send_request(method)
            except (EOFError, KeyboardInterrupt):
                break
    else:
        client.run_test_sequence()

if __name__ == "__main__":
    main()