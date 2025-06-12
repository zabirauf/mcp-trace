# MCP Proxy TUI (Rust)

A Terminal User Interface (TUI) for monitoring and proxying Model Context Protocol (MCP) servers, built with Rust and Ratatui.

## Architecture

This project consists of two main components:

### 1. MCP Proxy (`mcp-proxy`)
- **Purpose**: Acts as a transparent STDIO proxy for MCP servers
- **Functionality**:
  - Intercepts and logs all JSON-RPC communication
  - Forwards requests/responses between client and MCP server
  - Sends real-time logs and statistics to the monitor
  - Supports any STDIO-based MCP server

### 2. MCP Monitor (`mcp-monitor`)
- **Purpose**: TUI for monitoring multiple MCP proxies
- **Functionality**:
  - Real-time display of logs from all connected proxies
  - Statistics dashboard (requests, errors, data transfer)
  - Proxy status monitoring
  - Log filtering and scrolling

### Communication
- **IPC**: Unix domain sockets for proxy ↔ monitor communication
- **Protocol**: JSON-serialized structured messages
- **Transport**: Line-delimited JSON over async streams

## Quick Start

### Using Docker (Recommended)

```bash
# Start the monitor TUI
./run.sh monitor

# In another terminal, start a proxy with your MCP server
./run.sh proxy python your_mcp_server.py

# Or start multiple proxies
./run.sh proxy node mcp-server.js
./run.sh proxy ./my-binary-server --config config.json
```

### Manual Build

```bash
# Build all binaries
cargo build --release

# Start monitor
./target/release/mcp-monitor --verbose

# Start proxy (in another terminal)
./target/release/mcp-proxy --name "My Server" --command python server.py --verbose
```

## Usage Examples

### Running with Python MCP Server

```bash
# Start monitor
./run.sh monitor

# Start proxy for Python server
./run.sh proxy python -m my_mcp_package.server
```

### Running with Node.js MCP Server

```bash
# Start proxy for Node.js server
./run.sh proxy node dist/index.js
```

### Running with Binary MCP Server

```bash
# Start proxy for compiled binary
./run.sh proxy ./my-mcp-server --config server-config.json
```

## TUI Controls

- `q`: Quit the monitor
- `c`: Clear all logs
- `r`: Refresh proxy connections
- `↑/↓`: Scroll through logs
- `PgUp/PgDn`: Page up/down through logs
- `Home/End`: Jump to top/bottom of logs

## Configuration

### MCP Proxy Options

```bash
mcp-proxy [OPTIONS] --command <COMMAND>...

Options:
  -c, --command <COMMAND>...     MCP server command to proxy
  -n, --name <NAME>             Name for this proxy instance [default: mcp-proxy]
  -i, --ipc-socket <IPC_SOCKET> IPC socket path [default: /tmp/mcp-monitor.sock]
  -v, --verbose                 Verbose logging
```

### MCP Monitor Options

```bash
mcp-monitor [OPTIONS]

Options:
  -i, --ipc-socket <IPC_SOCKET> IPC socket path [default: /tmp/mcp-monitor.sock]
  -v, --verbose                 Verbose logging
```

## Project Structure

```
tool-mcp-proxy-tui/
├── mcp-common/          # Shared types and IPC communication
│   ├── src/
│   │   ├── types.rs     # Data structures
│   │   ├── messages.rs  # IPC message protocol
│   │   ├── ipc.rs       # Unix socket communication
│   │   └── mcp.rs       # JSON-RPC message handling
├── mcp-proxy/           # STDIO proxy binary
│   ├── src/
│   │   ├── main.rs      # CLI and application entry
│   │   ├── proxy.rs     # Main proxy logic
│   │   └── stdio_handler.rs # STDIO communication
├── mcp-monitor/         # TUI monitor binary
│   ├── src/
│   │   ├── main.rs      # TUI application setup
│   │   ├── app.rs       # Application state and logic
│   │   └── ui.rs        # Ratatui interface components
├── Dockerfile           # Multi-stage build
├── docker-compose.yml   # Service orchestration
└── run.sh              # Convenience script
```

## Features

### Proxy Features
- ✅ STDIO-based MCP server proxying
- ✅ Real-time JSON-RPC message logging
- ✅ Request/response statistics tracking
- ✅ Error handling and process management
- ✅ IPC communication with monitor

### Monitor Features
- ✅ Real-time TUI with Ratatui
- ✅ Multiple proxy monitoring
- ✅ Scrollable log viewer with syntax highlighting
- ✅ Statistics dashboard
- ✅ Proxy status indicators
- ✅ Keyboard navigation

### Communication
- ✅ Unix domain socket IPC
- ✅ Structured JSON message protocol
- ✅ Async message handling
- ✅ Connection management

## Development

### Building

```bash
# Build all workspace members
cargo build

# Build with optimizations
cargo build --release

# Run tests
cargo test
```

### Docker Development

```bash
# Build images
./run.sh build

# View logs
./run.sh logs

# Clean up
./run.sh clean
```

## Troubleshooting

### Common Issues

1. **Monitor not receiving proxy logs**
   - Ensure both proxy and monitor use the same IPC socket path
   - Check that `/tmp/mcp-sockets` directory is writable
   - Verify network connectivity in Docker environment

2. **Proxy fails to start MCP server**
   - Verify the command path and arguments
   - Check that the MCP server supports STDIO mode
   - Review proxy logs for detailed error messages

3. **TUI display issues**
   - Set `TERM=xterm-256color` environment variable
   - Ensure terminal supports 256 colors
   - Try running with `--verbose` for debugging

### Logs

- **Container logs**: `docker compose logs -f`
- **File logs**: Check `./logs/` directory
- **Proxy logs**: Include STDIO communication and errors
- **Monitor logs**: Include TUI events and IPC messages

## Contributing

1. Follow Rust conventions and use `cargo fmt`
2. Add tests for new functionality
3. Update documentation for API changes
4. Test with multiple MCP server implementations

## License

This project follows the same license as the original Python implementation.