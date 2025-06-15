#!/usr/bin/env python3
"""
Test MCP Server for testing the MCP Proxy and Monitor

This server simulates a real MCP server by:
- Reading JSON-RPC requests from stdin
- Adding configurable delays to responses
- Sending back proper JSON-RPC responses
- Generating various types of requests (tools, resources, prompts)
- Simulating different response scenarios (success, error, long responses)
"""

import sys
import json
import random
import asyncio
import argparse
from datetime import datetime
from typing import Dict, Any

class TestMCPServer:
    def __init__(self, delay_min: float = 0.5, delay_max: float = 3.0, error_rate: float = 0.1):
        self.delay_min = delay_min
        self.delay_max = delay_max
        self.error_rate = error_rate
        self.request_count = 0
        self.tools = [
            "calculator",
            "file_reader", 
            "web_search",
            "database_query",
            "email_sender"
        ]
        self.resources = [
            "config://settings.json",
            "file://documents/readme.md",
            "url://api.example.com/data",
            "database://users/table"
        ]
        
    def log(self, message: str):
        """Log to stderr so it doesn't interfere with JSON-RPC on stdout"""
        print(f"[{datetime.now().strftime('%H:%M:%S')}] {message}", file=sys.stderr)
        
    async def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle an incoming JSON-RPC request"""
        self.request_count += 1
        method = request.get("method", "unknown")
        request_id = request.get("id")
        
        self.log(f"Request #{self.request_count}: {method} (id: {request_id})")
        
        # Simulate processing delay
        delay = random.uniform(self.delay_min, self.delay_max)
        self.log(f"Processing for {delay:.2f}s...")
        await asyncio.sleep(delay)
        
        # Simulate random errors
        if random.random() < self.error_rate:
            return self.create_error_response(request_id, method)
            
        # Handle different MCP methods
        if method == "initialize":
            return self.handle_initialize(request_id)
        elif method == "tools/list":
            return self.handle_tools_list(request_id)
        elif method == "tools/call":
            return self.handle_tools_call(request_id, request.get("params", {}))
        elif method == "resources/list":
            return self.handle_resources_list(request_id)
        elif method == "resources/read":
            return self.handle_resources_read(request_id, request.get("params", {}))
        elif method == "prompts/list":
            return self.handle_prompts_list(request_id)
        elif method == "prompts/get":
            return self.handle_prompts_get(request_id, request.get("params", {}))
        else:
            return self.create_error_response(request_id, method, "Method not found")
    
    def handle_initialize(self, request_id) -> Dict[str, Any]:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {"listChanged": True},
                    "resources": {"subscribe": True, "listChanged": True},
                    "prompts": {"listChanged": True}
                },
                "serverInfo": {
                    "name": "test-mcp-server",
                    "version": "1.0.0"
                }
            }
        }
    
    def handle_tools_list(self, request_id) -> Dict[str, Any]:
        tools = []
        for tool_name in self.tools:
            tools.append({
                "name": tool_name,
                "description": f"A test {tool_name.replace('_', ' ')} tool",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "input": {"type": "string", "description": "Input for the tool"}
                    },
                    "required": ["input"]
                }
            })
        
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {"tools": tools}
        }
    
    def handle_tools_call(self, request_id, params: Dict[str, Any]) -> Dict[str, Any]:
        tool_name = params.get("name", "unknown")
        arguments = params.get("arguments", {})
        
        # Simulate different tool responses
        if tool_name == "calculator":
            result = f"Calculated result: {random.randint(1, 1000)}"
        elif tool_name == "web_search":
            result = f"Found {random.randint(5, 50)} search results for: {arguments.get('input', 'query')}"
        elif tool_name == "file_reader":
            result = f"File content: Lorem ipsum dolor sit amet, consectetur adipiscing elit..." + "x" * random.randint(100, 2000)
        else:
            result = f"Tool {tool_name} executed successfully with input: {arguments.get('input', 'N/A')}"
        
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": result
                    }
                ]
            }
        }
    
    def handle_resources_list(self, request_id) -> Dict[str, Any]:
        resources = []
        for resource_uri in self.resources:
            resources.append({
                "uri": resource_uri,
                "name": resource_uri.split("://")[-1],
                "description": f"Test resource: {resource_uri}",
                "mimeType": "text/plain"
            })
        
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {"resources": resources}
        }
    
    def handle_resources_read(self, request_id, params: Dict[str, Any]) -> Dict[str, Any]:
        uri = params.get("uri", "unknown://resource")
        
        # Generate different types of content
        content_types = ["json", "text", "long_text"]
        content_type = random.choice(content_types)
        
        if content_type == "json":
            content = json.dumps({
                "message": "This is a JSON resource",
                "timestamp": datetime.now().isoformat(),
                "data": {
                    "items": [f"item_{i}" for i in range(random.randint(3, 10))],
                    "metadata": {"version": "1.0", "type": "test"}
                }
            }, indent=2)
        elif content_type == "long_text":
            content = "This is a very long text resource. " + "Lorem ipsum dolor sit amet, consectetur adipiscing elit. " * random.randint(20, 100)
        else:
            content = f"Simple text content for resource: {uri}"
        
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "contents": [
                    {
                        "uri": uri,
                        "mimeType": "text/plain",
                        "text": content
                    }
                ]
            }
        }
    
    def handle_prompts_list(self, request_id) -> Dict[str, Any]:
        prompts = [
            {
                "name": "summarize",
                "description": "Summarize the given text",
                "arguments": [
                    {
                        "name": "text",
                        "description": "Text to summarize",
                        "required": True
                    }
                ]
            },
            {
                "name": "translate",
                "description": "Translate text between languages",
                "arguments": [
                    {
                        "name": "text",
                        "description": "Text to translate",
                        "required": True
                    },
                    {
                        "name": "target_language",
                        "description": "Target language",
                        "required": True
                    }
                ]
            }
        ]
        
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {"prompts": prompts}
        }
    
    def handle_prompts_get(self, request_id, params: Dict[str, Any]) -> Dict[str, Any]:
        prompt_name = params.get("name", "unknown")
        arguments = params.get("arguments", {})
        
        if prompt_name == "summarize":
            text = arguments.get("text", "No text provided")
            result = f"Summary: This text contains {len(text.split())} words and discusses various topics."
        elif prompt_name == "translate":
            text = arguments.get("text", "Hello")
            target = arguments.get("target_language", "Spanish")
            result = f"Translated to {target}: ¡Hola! (simulated translation)"
        else:
            result = f"Executed prompt '{prompt_name}' with arguments: {arguments}"
        
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "description": f"Result from {prompt_name} prompt",
                "messages": [
                    {
                        "role": "assistant",
                        "content": {
                            "type": "text",
                            "text": result
                        }
                    }
                ]
            }
        }
    
    def create_error_response(self, request_id, method: str, message: str = "Simulated error") -> Dict[str, Any]:
        error_codes = [-32000, -32001, -32002, -32003]  # Application-defined errors
        
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": random.choice(error_codes),
                "message": f"{message} in {method}",
                "data": {
                    "timestamp": datetime.now().isoformat(),
                    "method": method,
                    "details": "This is a simulated error for testing purposes"
                }
            }
        }
    
    async def run(self):
        """Main server loop"""
        self.log("Test MCP Server starting...")
        self.log(f"Response delay: {self.delay_min}-{self.delay_max}s, Error rate: {self.error_rate*100}%")
        
        try:
            while True:
                # Read line from stdin
                line = await asyncio.get_event_loop().run_in_executor(None, sys.stdin.readline)
                
                if not line:
                    self.log("EOF received, shutting down")
                    break
                
                line = line.strip()
                if not line:
                    continue
                
                try:
                    # Parse JSON-RPC request
                    request = json.loads(line)
                    
                    # Handle the request
                    response = await self.handle_request(request)
                    
                    # Send response to stdout
                    response_json = json.dumps(response, separators=(',', ':'))
                    print(response_json, flush=True)
                    
                    self.log(f"Sent response for {request.get('method', 'unknown')}")
                    
                except json.JSONDecodeError as e:
                    self.log(f"Invalid JSON received: {e}")
                    error_response = {
                        "jsonrpc": "2.0",
                        "id": None,
                        "error": {
                            "code": -32700,
                            "message": "Parse error",
                            "data": str(e)
                        }
                    }
                    print(json.dumps(error_response, separators=(',', ':')), flush=True)
                
                except Exception as e:
                    self.log(f"Error handling request: {e}")
                    error_response = {
                        "jsonrpc": "2.0",
                        "id": None,
                        "error": {
                            "code": -32603,
                            "message": "Internal error",
                            "data": str(e)
                        }
                    }
                    print(json.dumps(error_response, separators=(',', ':')), flush=True)
                    
        except KeyboardInterrupt:
            self.log("Received interrupt, shutting down")
        except Exception as e:
            self.log(f"Unexpected error: {e}")

def main():
    parser = argparse.ArgumentParser(description="Test MCP Server")
    parser.add_argument("--delay-min", type=float, default=0.5,
                        help="Minimum response delay in seconds (default: 0.5)")
    parser.add_argument("--delay-max", type=float, default=3.0,
                        help="Maximum response delay in seconds (default: 3.0)")
    parser.add_argument("--error-rate", type=float, default=0.1,
                        help="Error rate (0.0-1.0, default: 0.1)")
    parser.add_argument("--fast", action="store_true",
                        help="Fast mode: 0.1-0.5s delays, no errors")
    parser.add_argument("--slow", action="store_true",
                        help="Slow mode: 2-8s delays, 20 percent errors")
    
    args = parser.parse_args()
    
    # Apply presets
    if args.fast:
        args.delay_min, args.delay_max, args.error_rate = 0.1, 0.5, 0.0
    elif args.slow:
        args.delay_min, args.delay_max, args.error_rate = 2.0, 8.0, 0.2
    
    server = TestMCPServer(args.delay_min, args.delay_max, args.error_rate)
    asyncio.run(server.run())

if __name__ == "__main__":
    main()