#!/usr/bin/env python3
"""
MCP Proxy TUI - A Terminal User Interface for proxying and monitoring MCP server traffic
"""

from textual.app import App, ComposeResult
from textual.containers import Container, Horizontal, Vertical, ScrollableContainer
from textual.widgets import Header, Footer, Static, ListView, ListItem, RichLog, Label, Button, Input
from textual.binding import Binding
from textual.reactive import reactive
from datetime import datetime
import asyncio
from typing import Dict, List, Optional
import json
import structlog
from pathlib import Path

# Configure structured logging
logger = structlog.get_logger()

class LogEntry(ListItem):
    """A single log entry in the log viewer"""
    
    def __init__(self, timestamp: str, level: str, message: str, details: Optional[Dict] = None):
        super().__init__()
        self.timestamp = timestamp
        self.level = level
        self.message = message
        self.details = details or {}
        
    def compose(self) -> ComposeResult:
        level_colors = {
            "INFO": "cyan",
            "WARNING": "yellow",
            "ERROR": "red",
            "DEBUG": "green",
            "REQUEST": "blue",
            "RESPONSE": "magenta"
        }
        color = level_colors.get(self.level, "white")
        
        yield Static(
            f"[dim]{self.timestamp}[/dim] [{color}]{self.level:8}[/{color}] {self.message}",
            classes="log-entry"
        )

class MCPConnection(Static):
    """Widget showing MCP server connection status"""
    
    def __init__(self, server_name: str, server_url: str):
        super().__init__()
        self.server_name = server_name
        self.server_url = server_url
        self.status = "disconnected"
        self.request_count = 0
        self.error_count = 0
        
    def compose(self) -> ComposeResult:
        status_symbol = "🔴" if self.status == "disconnected" else "🟢"
        yield Static(
            f"{status_symbol} {self.server_name} - {self.server_url}\n"
            f"   Requests: {self.request_count} | Errors: {self.error_count}"
        )

class MCPProxyTUI(App):
    """Main TUI Application for MCP Proxy"""
    
    CSS = """
    Screen {
        layout: grid;
        grid-size: 2 3;
        grid-rows: 1fr 3fr 1fr;
        grid-columns: 1fr 2fr;
    }
    
    #sidebar {
        dock: left;
        width: 40;
        border: solid green;
        overflow-y: auto;
    }
    
    #main-content {
        border: solid blue;
    }
    
    #log-viewer {
        height: 100%;
        border: solid yellow;
        overflow-y: auto;
    }
    
    #stats {
        height: 8;
        border: solid magenta;
        padding: 1;
    }
    
    .log-entry {
        padding: 0 1;
    }
    
    MCPConnection {
        height: 3;
        margin: 1;
        padding: 1;
        border: solid white;
    }
    
    #command-input {
        dock: bottom;
        height: 3;
    }
    """
    
    BINDINGS = [
        Binding("q", "quit", "Quit"),
        Binding("c", "clear_logs", "Clear Logs"),
        Binding("r", "refresh", "Refresh"),
        Binding("a", "add_server", "Add Server"),
        Binding("d", "toggle_dark", "Toggle Dark Mode"),
    ]
    
    def __init__(self):
        super().__init__()
        self.mcp_servers: Dict[str, MCPConnection] = {}
        self.log_entries: List[LogEntry] = []
        self.proxy_port = 8765
        self.log_file = Path("/app/logs/mcp_proxy.log")
        self.log_file.parent.mkdir(parents=True, exist_ok=True)
        
    def compose(self) -> ComposeResult:
        """Create the UI layout"""
        yield Header(show_clock=True)
        
        with Container(id="sidebar"):
            yield Label("MCP Servers", classes="section-title")
            yield ScrollableContainer(id="server-list")
            yield Button("Add Server", id="add-server-btn", variant="primary")
            
        with Vertical(id="main-content"):
            yield Label("Request/Response Logs", classes="section-title")
            yield RichLog(id="log-viewer", highlight=True, markup=True)
            
        with Container(id="stats"):
            yield Label("Statistics", classes="section-title")
            yield Static("Total Requests: 0 | Total Errors: 0 | Uptime: 00:00:00", id="stats-display")
            
        yield Footer()
    
    def on_mount(self) -> None:
        """Initialize the app when mounted"""
        self.log_info("MCP Proxy TUI Started", "Application initialized")
        self.set_interval(1.0, self.update_stats)
        
        # Start the proxy server
        asyncio.create_task(self.start_proxy_server())
        
        # Add some demo servers
        self.add_mcp_server("Local Dev", "http://localhost:3000")
        self.add_mcp_server("Test Server", "http://localhost:8000")
    
    def log_info(self, message: str, details: str = "") -> None:
        """Add an info log entry"""
        self.add_log_entry("INFO", message, {"details": details})
    
    def log_error(self, message: str, error: str = "") -> None:
        """Add an error log entry"""
        self.add_log_entry("ERROR", message, {"error": error})
    
    def log_request(self, server: str, method: str, path: str, body: Optional[Dict] = None) -> None:
        """Log an MCP request"""
        self.add_log_entry("REQUEST", f"{server}: {method} {path}", {"body": body})
    
    def log_response(self, server: str, status: int, body: Optional[Dict] = None) -> None:
        """Log an MCP response"""
        self.add_log_entry("RESPONSE", f"{server}: Status {status}", {"body": body})
    
    def add_log_entry(self, level: str, message: str, details: Optional[Dict] = None) -> None:
        """Add a log entry to the viewer"""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        entry = LogEntry(timestamp, level, message, details)
        self.log_entries.append(entry)
        
        # Write to log file
        log_data = {
            "timestamp": timestamp,
            "level": level,
            "message": message,
            "details": details
        }
        with open(self.log_file, "a") as f:
            f.write(json.dumps(log_data) + "\n")
        
        # Update UI
        log_viewer = self.query_one("#log-viewer", RichLog)
        # Format the log entry for RichLog
        level_colors = {
            "INFO": "cyan",
            "WARNING": "yellow",
            "ERROR": "red",
            "DEBUG": "green",
            "REQUEST": "blue",
            "RESPONSE": "magenta"
        }
        color = level_colors.get(level, "white")
        log_viewer.write(f"[dim]{timestamp}[/dim] [{color}]{level:8}[/{color}] {message}")
    
    def add_mcp_server(self, name: str, url: str) -> None:
        """Add an MCP server to monitor"""
        connection = MCPConnection(name, url)
        self.mcp_servers[name] = connection
        
        server_list = self.query_one("#server-list", ScrollableContainer)
        server_list.mount(connection)
        
        self.log_info(f"Added MCP Server: {name}", f"URL: {url}")
    
    async def start_proxy_server(self) -> None:
        """Start the proxy server to intercept MCP traffic"""
        # This is a placeholder for the actual proxy implementation
        self.log_info("Proxy Server Started", f"Listening on port {self.proxy_port}")
        
        # Simulate some traffic for demo
        await asyncio.sleep(2)
        self.log_request("Local Dev", "POST", "/v1/messages", {"content": "Hello MCP"})
        await asyncio.sleep(0.5)
        self.log_response("Local Dev", 200, {"response": "Message received"})
        
        await asyncio.sleep(3)
        self.log_request("Test Server", "GET", "/v1/tools", None)
        await asyncio.sleep(0.5)
        self.log_response("Test Server", 200, {"tools": ["calculator", "web_search"]})
        
        await asyncio.sleep(2)
        self.log_error("Connection Error", "Failed to connect to Test Server")
    
    def update_stats(self) -> None:
        """Update statistics display"""
        stats = self.query_one("#stats-display", Static)
        total_requests = sum(s.request_count for s in self.mcp_servers.values())
        total_errors = sum(s.error_count for s in self.mcp_servers.values())
        
        # Calculate uptime (simplified)
        uptime = datetime.now() - datetime.now()  # Replace with actual start time
        uptime_str = "00:00:00"  # Format properly in production
        
        stats.update(f"Total Requests: {total_requests} | Total Errors: {total_errors} | Uptime: {uptime_str}")
    
    def action_clear_logs(self) -> None:
        """Clear all log entries"""
        self.log_entries.clear()
        log_viewer = self.query_one("#log-viewer", RichLog)
        log_viewer.clear()
        self.log_info("Logs Cleared", "All log entries have been removed")
    
    def action_refresh(self) -> None:
        """Refresh server connections"""
        self.log_info("Refreshing Connections", "Checking all MCP server connections")
        # Implement actual refresh logic
    
    def action_add_server(self) -> None:
        """Show dialog to add new server"""
        # In a real implementation, this would show an input dialog
        self.log_info("Add Server", "Feature coming soon!")

def main():
    """Entry point for the application"""
    app = MCPProxyTUI()
    app.run()

if __name__ == "__main__":
    main()