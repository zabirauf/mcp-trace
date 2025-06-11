# MCP Proxy TUI

A Terminal User Interface (TUI) application for proxying and monitoring Model Context Protocol (MCP) server traffic. Built with Python and Textual for a modern, responsive terminal experience.

## Features

- **Real-time MCP Traffic Monitoring**: Intercept and display all requests/responses between MCP clients and servers
- **Multiple Server Support**: Monitor multiple MCP servers simultaneously
- **Structured Logging**: All traffic is logged with timestamps and structured data
- **Interactive TUI**: Modern terminal interface with keyboard shortcuts and mouse support
- **Statistics Dashboard**: Track request counts, errors, and uptime
- **Log Persistence**: All logs are saved to disk for later analysis

## Architecture

The application acts as a proxy between MCP clients and servers:

```
MCP Client → MCP Proxy TUI → MCP Server
           ↓
      Logging & Display
```

## Usage

### Running with Docker

```bash
./run.sh
```

Or manually:

```bash
docker compose up --build
```

### Keyboard Shortcuts

- `q` - Quit the application
- `c` - Clear all logs
- `r` - Refresh server connections
- `a` - Add a new MCP server
- `d` - Toggle dark/light mode

## Project Structure

```
tool-mcp-proxy-tui/
├── src/
│   ├── __init__.py
│   └── main.py          # Main TUI application
├── logs/                # Persisted log files
├── Dockerfile
├── docker-compose.yml
├── requirements.txt
├── run.sh
└── README.md
```

## Configuration

The proxy listens on port 8765 by default. MCP servers can be added through the UI or configured in the application.

## Log Format

Logs are stored in JSON format at `/app/logs/mcp_proxy.log`:

```json
{
  "timestamp": "14:32:15.123",
  "level": "REQUEST",
  "message": "Local Dev: POST /v1/messages",
  "details": {
    "body": {"content": "Hello MCP"}
  }
}
```

## Development

The application uses:
- **Textual**: Modern TUI framework for Python
- **Rich**: Terminal formatting and styling
- **structlog**: Structured logging
- **asyncio**: Asynchronous networking

## Future Enhancements

- WebSocket support for real-time MCP connections
- Request/response filtering and search
- Export logs in various formats
- Performance metrics and latency tracking
- Request replay functionality
- SSL/TLS support for secure connections