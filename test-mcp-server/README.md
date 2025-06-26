# Test MCP Server

This directory contains a test MCP server and utilities for testing the MCP Proxy and Monitor functionality.

## Files

- **`test_server.py`** - Simulated MCP server with configurable delays and error rates
- **`test_client.py`** - Test client that sends various MCP requests  
- **`run_tests.sh`** - Convenience script for running tests
- **`README.md`** - This file

## Quick Start

### 1. Complete Test Setup

```bash
# Terminal 1: Start monitor
./run_tests.sh monitor

# Terminal 2: Start proxy with test server  
./run_tests.sh proxy --fast

# Terminal 3: Generate test traffic
./run_tests.sh client
```

### 2. Manual Testing

```bash
# Start test server directly
python3 test_server.py --fast

# Start test client
python3 test_client.py
```

### 3. Using with mcp-trace

```bash
# Start monitor
mcp-trace monitor

# In another terminal, start proxy pointing to test server
mcp-trace proxy --command "python3 test-mcp-server/test_server.py --fast" --name "TestServer"
```

## Test Server Options

The test server simulates realistic MCP behavior with configurable timing:

```bash
python3 test_server.py [OPTIONS]

Options:
  --delay-min FLOAT     Minimum response delay in seconds (default: 0.5)
  --delay-max FLOAT     Maximum response delay in seconds (default: 3.0) 
  --error-rate FLOAT    Error rate 0.0-1.0 (default: 0.1)
  --fast                Fast mode: 0.1-0.5s delays, no errors
  --slow                Slow mode: 2-8s delays, 20% errors
```

### Presets

- **`--fast`**: Quick responses (0.1-0.5s), no errors - good for UI testing
- **`--slow`**: Slow responses (2-8s), 20% errors - stress testing
- **Default**: Medium responses (0.5-3s), 10% errors - realistic simulation

## What the Test Server Simulates

The server implements realistic MCP protocol behavior:

### Supported Methods
- **`initialize`** - Protocol handshake
- **`tools/list`** - List available tools (calculator, web_search, file_reader, etc.)
- **`tools/call`** - Execute tools with various response sizes
- **`resources/list`** - List available resources
- **`resources/read`** - Read resources with different content types (JSON, text, long text)
- **`prompts/list`** - List available prompts
- **`prompts/get`** - Execute prompts

### Response Variations
- **JSON responses** - Properly formatted JSON-RPC responses
- **Variable content** - Different response sizes and types
- **Realistic delays** - Configurable processing time simulation
- **Error simulation** - Random errors to test error handling
- **Long responses** - Large payloads to test UI scrolling/formatting

## Test Client Features

The test client sends a realistic sequence of MCP requests:

```bash
python3 test_client.py [--interactive]

# Interactive mode for manual testing
python3 test_client.py --interactive
```

### Test Sequence
1. Initialize protocol
2. List and call various tools
3. List and read different resources  
4. List and execute prompts
5. Random timing between requests

## Integration with MCP Proxy/Monitor

This test setup is perfect for testing the MCP debugging tools:

### Monitor Features Tested
- **Tab filtering** - See different log types in separate tabs
- **Real-time updates** - Watch requests/responses as they happen
- **Detail view** - Press Enter on requests/responses to see formatted JSON
- **Word wrap** - Toggle word wrapping in detail view with 'W'
- **Connection tracking** - See proxy connection/disconnection events

### Proxy Features Tested  
- **STDIO forwarding** - Transparent request/response passing
- **Error handling** - Proper handling of server errors
- **Performance** - Handling of delayed responses
- **Protocol compliance** - Correct JSON-RPC message handling

## Example Usage

### Fast Development Testing
```bash
# Quick feedback loop
./run_tests.sh proxy --fast
```

### Realistic Testing
```bash
# Normal timing with some errors
./run_tests.sh proxy  
```

### Stress Testing
```bash  
# Slow responses with many errors
./run_tests.sh proxy --slow
```

### Interactive Testing
```bash
# Manual request sending
./run_tests.sh client --interactive
```

This test server provides a comprehensive way to exercise all the features of the MCP Proxy and Monitor without needing a real MCP server.