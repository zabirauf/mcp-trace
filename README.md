# MCP Probe

A powerful Terminal User Interface (TUI) for monitoring and debugging Model Context Protocol (MCP) servers in real-time. Built with Rust and designed for MCP developers who need deep visibility into their server implementations.

## Why MCP Probe?

Developing MCP servers can be challenging when you can't see what's happening under the hood. MCP Probe solves this by providing:

- **Real-time monitoring** of all JSON-RPC communication between clients and servers
- **Multiple server tracking** - monitor several MCP servers simultaneously
- **Request/response analysis** with detailed statistics and error tracking
- **Zero-configuration setup** - works with any STDIO-based MCP server
- **Beautiful TUI interface** with syntax highlighting and intuitive navigation

Perfect for:
- **Learning MCP** by observing real communication flows
- **MCP server developers** debugging their implementations

## Quick Start

### Using Docker (Recommended)

```bash
# Clone the repository
git clone https://github.com/zabirauf/mcp-probe
cd mcp-probe

# Start the monitoring interface
./run.sh monitor

# In another terminal, start monitoring your MCP server
./run.sh proxy python your_mcp_server.py
```

### Install from Source

```bash
# Build the project
cargo build --release

# Start the monitor
./target/release/mcp-monitor

# Start monitoring your server (in another terminal)
./target/release/mcp-proxy --name "My Server" --command python your_server.py
```

## Usage Examples

### Python MCP Server

```bash
# Monitor a Python MCP server
./run.sh proxy python -m my_mcp_package.server

# With arguments
./run.sh proxy python server.py --config config.json --port 8080
```

### Node.js MCP Server

```bash
# Monitor a Node.js MCP server
./run.sh proxy node dist/server.js

# With npm script
./run.sh proxy npm run start:mcp
```

### Binary MCP Server

```bash
# Monitor a compiled binary server
./run.sh proxy ./my-mcp-server --verbose

# With custom configuration
./run.sh proxy ./server --config /path/to/config.toml
```

### Multiple Servers

Monitor multiple MCP servers simultaneously:

```bash
# Terminal 1: Start monitor
./run.sh monitor

# Terminal 2: Start first server
./run.sh proxy python server1.py

# Terminal 3: Start second server  
./run.sh proxy node server2.js

# Terminal 4: Start third server
./run.sh proxy ./binary-server --config prod.json
```

## TUI Interface

### Main View
- **Left Panel**: List of connected servers with status indicators
- **Right Panel**: Real-time log viewer with JSON syntax highlighting
- **Bottom Panel**: Statistics dashboard and help text

### Navigation
- `q` - Quit the application
- `c` - Clear all logs
- `r` - Refresh connections
- `↑/↓` - Scroll through logs
- `PgUp/PgDn` - Page up/down
- `Home/End` - Jump to top/bottom
- `Tab` - Switch between panels

### Status Indicators
- 🟢 **Connected** - Server is running and responding
- 🟡 **Starting** - Server is initializing
- 🔴 **Error** - Server encountered an error
- ⚫ **Disconnected** - Server is not responding

## Command Reference

### mcp-probe monitor

Start the monitoring TUI interface.

```bash
mcp-probe monitor [OPTIONS]

Options:
  -v, --verbose                 Enable verbose logging
  -s, --socket <PATH>          Custom IPC socket path
  -h, --help                   Show help information
```

### mcp-probe proxy

Start monitoring an MCP server.

```bash
mcp-probe proxy [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>                    Command to start your MCP server

Options:
  -n, --name <NAME>           Display name for this server
  -s, --socket <PATH>         IPC socket path to connect to monitor
  -v, --verbose               Enable verbose logging
  -h, --help                  Show help information

Examples:
  mcp-probe proxy python server.py
  mcp-probe proxy --name "File Server" node dist/file-server.js
  mcp-probe proxy --verbose ./my-binary-server --config config.json
```

## What You'll See

### Request/Response Logging
Every JSON-RPC message is captured and displayed with:
- Timestamp and direction (→ outgoing, ← incoming)
- Method names and IDs for easy tracking
- Full JSON payloads with syntax highlighting
- Error messages and stack traces

### Statistics Dashboard
- **Total Requests** - Number of requests processed
- **Success Rate** - Percentage of successful requests
- **Average Response Time** - Performance metrics
- **Data Transfer** - Bytes sent/received
- **Active Connections** - Current client connections

### Error Tracking
- Real-time error highlighting
- Stack trace preservation
- Error categorization (client vs server errors)
- Request/response correlation for debugging

## Installation

### Prerequisites
- Rust 1.70+ (if building from source)
- Docker (for containerized usage)
- Your MCP server implementation

### From Source
```bash
git clone https://github.com/zabirauf/mcp-probe
cd mcp-probe
cargo install --path .
```
## Configuration

### Environment Variables
- `RUST_LOG` - Set logging level (debug, info, warn, error)
- `MCP_SOCKET_PATH` - Custom IPC socket location
- `TERM` - Terminal type (recommended: xterm-256color)

### Socket Configuration
By default, MCP Probe uses Unix domain sockets at `/tmp/mcp-monitor.sock`. You can customize this:

```bash
# Custom socket path
mcp-probe monitor --socket /custom/path/monitor.sock
mcp-probe proxy --socket /custom/path/monitor.sock python server.py
```

## Troubleshooting

### Common Issues

**Monitor shows no connections**
- Ensure both monitor and proxy use the same socket path
- Check that the socket directory is writable
- Verify the proxy command is correct

**Server fails to start**
- Verify your MCP server command works independently
- Check that the server supports STDIO mode
- Review proxy logs with `--verbose` flag

**TUI display problems**
- Set `TERM=xterm-256color`
- Ensure terminal supports 256 colors
- Try resizing the terminal window

### Debug Mode
Enable verbose logging to see detailed information:

```bash
RUST_LOG=debug mcp-probe monitor --verbose
RUST_LOG=debug mcp-probe proxy --verbose python server.py
```

## Contributing

We welcome contributions! This project is built with an AI-first approach where Claude and other AI assistants are primary development partners.

### Development Setup
```bash
git clone https://github.com/zabirauf/mcp-probe
cd mcp-probe
cargo build
cargo test
```

### Architecture
MCP Probe consists of two main components that communicate via IPC:
- **mcp-probe monitor** - The TUI interface
- **mcp-probe proxy** - The transparent proxy that intercepts MCP traffic

## AI-First Development

This project embraces AI-assisted development as a core philosophy. We believe that:

- **AI partnerships accelerate development** - Claude and other AI assistants are treated as primary development partners, not just tools
- **Human creativity + AI efficiency** - Combining human insight with AI's rapid iteration capabilities produces better software faster
- **Documentation-driven development** - Comprehensive documentation (like CLAUDE.md) enables AI assistants to understand and contribute meaningfully to the codebase

The codebase includes detailed AI-guidance documentation and is structured to be easily understood and extended by AI assistants. We encourage contributors to embrace this collaborative approach and document their code in ways that facilitate AI understanding and contribution.

Future development will continue to leverage AI partnerships for feature development, testing, documentation, and optimization. We see this as the future of open-source development and invite the community to explore and contribute to this paradigm.

## License

MIT License - see LICENSE file for details.
