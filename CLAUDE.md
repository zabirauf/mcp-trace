# CLAUDE.md

This file provides guidance to Claude Code when working with the MCP Proxy TUI project.

## Project Overview

This is a Terminal User Interface (TUI) application built with Python and Textual that acts as a proxy and monitor for Model Context Protocol (MCP) server traffic. It intercepts, logs, and displays all communication between MCP clients and servers in real-time.

## Key Components

### Main Application (`src/main.py`)
- **MCPProxyTUI**: The main Textual app class that manages the UI and proxy logic
- **LogEntry**: Widget for displaying individual log entries with formatting
- **MCPConnection**: Widget showing connection status for each MCP server
- Uses CSS-in-Python for styling the TUI layout

### Architecture
```
MCP Client → [Proxy Layer] → MCP Server
                 ↓
           [TUI Display]
                 ↓
           [Log Storage]
```

## Development Guidelines

### Adding New Features
1. **New UI Components**: Extend Textual widgets in `src/main.py`
2. **Proxy Logic**: Implement in async methods within MCPProxyTUI class
3. **Logging**: Use the structured logging methods (log_info, log_error, log_request, log_response)

### Code Patterns
- Use async/await for all network operations
- Follow Textual's reactive programming model
- Keep UI updates in the main thread
- Use structured logging with appropriate levels

### Testing the TUI
```bash
# Run in development mode with live reload
docker compose up --build

# View logs
docker compose logs -f

# Access the container for debugging
docker compose exec mcp-proxy-tui bash
```

## Common Tasks

### Adding a New MCP Server
```python
self.add_mcp_server("Server Name", "http://server-url:port")
```

### Implementing Actual Proxy Logic
Replace the placeholder in `start_proxy_server()` with:
1. HTTP/WebSocket server listening on proxy_port
2. Request interception and forwarding
3. Response capture and forwarding
4. Error handling and retry logic

### Adding New Log Types
1. Create new log method in MCPProxyTUI class
2. Define color in LogEntry.compose() level_colors dict
3. Update stats tracking if needed

## UI Layout

The TUI is divided into three main sections:
- **Sidebar** (left): MCP server list and controls
- **Main Content** (center): Log viewer with scrolling
- **Stats Bar** (bottom): Real-time statistics

## Keyboard Bindings

Current bindings defined in BINDINGS:
- `q`: Quit application
- `c`: Clear logs
- `r`: Refresh connections
- `a`: Add new server
- `d`: Toggle dark mode

To add new bindings:
1. Add to BINDINGS list
2. Implement `action_<name>` method

## Dependencies

Key libraries:
- **textual**: TUI framework
- **rich**: Terminal formatting
- **structlog**: Structured logging
- **aiohttp/httpx**: HTTP client/server
- **websockets**: WebSocket support
- **mcp**: Model Context Protocol library

## Performance Considerations

- Log entries are appended to both UI and disk
- Consider implementing log rotation for long-running sessions
- Use pagination or virtualization for large log volumes
- Implement connection pooling for multiple MCP servers

## Future Implementation Notes

### WebSocket Support
```python
async def handle_websocket(self, ws, path):
    # Implement bidirectional WebSocket proxy
    pass
```

### Request Filtering
```python
def filter_requests(self, pattern: str):
    # Implement log filtering logic
    pass
```

### SSL/TLS Support
- Add certificate handling in docker-compose.yml
- Implement SSL context in proxy server

## Debugging Tips

1. Check `/app/logs/mcp_proxy.log` for persisted logs
2. Use `structlog` configuration for detailed debugging
3. Enable Textual dev tools with `--dev` flag
4. Monitor Docker logs for startup issues

## Container Configuration

- Runs with TTY enabled for proper TUI rendering
- Network mode is 'host' for easy MCP server access
- Volume mounts for live code updates and log persistence
- Environment variables for terminal colors