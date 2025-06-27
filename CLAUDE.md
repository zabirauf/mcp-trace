# CLAUDE.md

This file provides guidance to Claude Code when working with the MCP Proxy TUI project.
## Project Overview

This is a Terminal User Interface (TUI) application built with **Rust and Ratatui** that acts as a proxy and monitor for Model Context Protocol (MCP) server traffic. It consists of two separate binaries that work together to intercept, log, and display all communication between MCP clients and servers in real-time.

## Architecture

The project is structured as a **Cargo workspace** with two main components:

### 1. MCP Proxy (`mcp-proxy`)
- **Purpose**: STDIO-based proxy for MCP servers
- **Language**: Rust with tokio for async operations
- **Functionality**: Intercepts and forwards JSON-RPC communication, sends logs to monitor
- **Binary location**: `target/release/mcp-proxy`

### 2. MCP Monitor (`mcp-monitor`) 
- **Purpose**: TUI for monitoring multiple MCP proxies
- **Language**: Rust with Ratatui for terminal UI
- **Functionality**: Real-time display of logs, statistics, and proxy status
- **Binary location**: `target/release/mcp-monitor`

### 3. Shared Library (`mcp-common`)
- **Purpose**: Common types, IPC communication, and message protocols
- **Components**: Types, IPC (Unix sockets), JSON-RPC handling

```
MCP Client → [mcp-proxy] → MCP Server (STDIO)
                 ↓ (IPC)
           [mcp-monitor TUI]
                 ↓
           [Log Storage]
```

## Repository Information

**GitHub Repository**: `zabirauf/mcp-trace`

## Binary Distribution

When setting up binary distribution, **only the `mcp-trace` binary should be distributed**. The other binaries (`mcp-monitor` and `mcp-proxy`) are for internal use only and should not be included in public distributions.

## Commit Message Style

Keep git commit messages concise and direct. Avoid verbose explanations.

## Branch Management

For new tasks unrelated to the current work, always create a new branch from `main`:
```bash
git checkout main
git pull origin main
git checkout -b feature/new-task-name
```

## Key Components

### MCP Proxy Components
- **`src/main.rs`**: CLI argument parsing and application entry point
- **`src/proxy.rs`**: Main proxy logic and MCP server process management
- **`src/stdio_handler.rs`**: STDIO communication handling between client and server

### MCP Monitor Components  
- **`src/main.rs`**: TUI application setup and IPC server
- **`src/app.rs`**: Application state management and event handling
- **`src/ui.rs`**: Ratatui interface components and layout

### MCP Common Components
- **`src/types.rs`**: Shared data structures (ProxyId, LogEntry, ProxyStats, etc.)
- **`src/messages.rs`**: IPC message protocol definitions
- **`src/ipc.rs`**: Unix domain socket communication
- **`src/mcp.rs`**: JSON-RPC message parsing and handling

## Development Guidelines

### Building and Running

```bash
# Build all components
cargo build --release

# Run monitor (starts IPC server)
./target/release/mcp-monitor --verbose

# Run proxy (in another terminal)
./target/release/mcp-proxy --name "My Server" --command python server.py --verbose

# Or use convenience script
./run.sh monitor
./run.sh proxy python server.py
```

### Adding New Features

#### 1. New Proxy Features
- **STDIO handling**: Modify `mcp-proxy/src/stdio_handler.rs`
- **Process management**: Update `mcp-proxy/src/proxy.rs`
- **New CLI options**: Add to `mcp-proxy/src/main.rs` clap Args struct

#### 2. New Monitor Features
- **UI components**: Extend widgets in `mcp-monitor/src/ui.rs`
- **Application logic**: Update state management in `mcp-monitor/src/app.rs`
- **Event handling**: Add new AppEvent variants and handlers

#### 3. New Communication Features
- **Message types**: Add to `mcp-common/src/messages.rs` IpcMessage enum
- **Data structures**: Define in `mcp-common/src/types.rs`
- **Protocol changes**: Update IPC handling in `mcp-common/src/ipc.rs`

### Code Patterns

#### Async/Await Usage
```rust
// All network operations use tokio
async fn handle_communication(&mut self) -> Result<()> {
    tokio::select! {
        result = self.read_stdin() => { /* handle */ }
        result = self.read_stdout() => { /* handle */ }
        _ = self.shutdown_rx.recv() => break,
    }
}
```

#### Error Handling
```rust
// Use anyhow for error propagation
use anyhow::{Result, Context};

async fn start_server(&self) -> Result<()> {
    Command::new(&self.command[0])
        .spawn()
        .context("Failed to start MCP server")?;
}
```

#### IPC Communication
```rust
// Structured message sending
let message = IpcMessage::LogEntry(log_entry);
ipc_client.send(message).await?;
```

### Testing and Development

```bash
# Check compilation
cargo check

# Run with logging
RUST_LOG=debug ./target/release/mcp-monitor

# Docker development
./run.sh build
docker compose up --build

# View logs
./run.sh logs
```

## Common Tasks

### Adding a New Log Type

1. **Add to LogLevel enum** in `mcp-common/src/types.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    // ... existing levels
    NewLevel,
}
```

2. **Update UI colors** in `mcp-monitor/src/ui.rs`:
```rust
let level_color = match log.level {
    // ... existing colors
    LogLevel::NewLevel => Color::Purple,
};
```

3. **Add logging method** in `mcp-proxy/src/stdio_handler.rs`:
```rust
async fn log_new_level(&mut self, content: &str) {
    let log_entry = LogEntry::new(
        LogLevel::NewLevel,
        content.to_string(),
        self.proxy_id.clone(),
    );
    // Send to monitor...
}
```

### Adding New IPC Messages

1. **Define message** in `mcp-common/src/messages.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    // ... existing messages
    NewMessage { data: String },
}
```

2. **Handle in monitor** in `mcp-monitor/src/app.rs`:
```rust
pub enum AppEvent {
    // ... existing events  
    NewEvent(String),
}

// In handle_event:
AppEvent::NewEvent(data) => {
    // Handle new event
}
```

### Adding New CLI Options

1. **Proxy options** in `mcp-proxy/src/main.rs`:
```rust
#[derive(Parser)]
pub struct Args {
    // ... existing args
    #[arg(long)]
    pub new_option: String,
}
```

2. **Monitor options** in `mcp-monitor/src/main.rs`:
```rust
#[derive(Parser)]  
pub struct Args {
    // ... existing args
    #[arg(long)]
    pub new_flag: bool,
}
```

## UI Layout and Controls

### TUI Layout
- **Left panel** (30 units): Proxy list with status indicators
- **Right panel** (main): Log viewer with scrolling
- **Bottom panel** (3 lines): Help text and keyboard shortcuts
- **Bottom of left panel** (8 lines): Statistics dashboard

### Keyboard Bindings

Current bindings in `mcp-monitor/src/main.rs`:
- `q`: Quit application
- `c`: Clear logs  
- `r`: Refresh connections
- `↑/↓`: Scroll through logs
- `PgUp/PgDn`: Page up/down through logs
- `Home/End`: Jump to top/bottom of logs

To add new bindings:
1. Handle in `run_app` key match statement
2. Implement corresponding method in `App`

## Dependencies

### Rust Crates
- **tokio**: Async runtime and networking
- **ratatui**: Terminal user interface framework
- **crossterm**: Cross-platform terminal manipulation
- **serde/serde_json**: Serialization
- **clap**: Command-line argument parsing
- **tracing**: Structured logging
- **anyhow/thiserror**: Error handling
- **uuid**: Unique identifiers
- **chrono**: Date/time handling

## Performance Considerations

- **Log management**: Limits to 10,000 entries, auto-scrolling
- **Memory usage**: Structured data with efficient serialization
- **IPC overhead**: Line-delimited JSON over Unix sockets
- **Async operations**: Non-blocking I/O throughout
- **Stats updates**: 1-second intervals to avoid overwhelming the UI

## Container Configuration

### Multi-stage Docker Build
- **Builder stage**: Rust compilation with all dependencies
- **Runtime stage**: Minimal Debian with just the binaries
- **User management**: Non-root app user for security
- **Volume mounts**: Logs and socket directories

### Docker Compose
- **mcp-monitor**: TUI service with TTY enabled
- **mcp-proxy-example**: Example proxy configuration
- **Networking**: Bridge network for inter-service communication
- **Environment**: RUST_LOG and terminal color support

## Debugging Tips

1. **Compilation issues**: Run `cargo check` for quick feedback
2. **Runtime logs**: Use `RUST_LOG=debug` for detailed tracing
3. **IPC issues**: Check `/tmp/mcp-sockets` permissions
4. **TUI problems**: Ensure `TERM=xterm-256color` is set
5. **Process issues**: Monitor `docker compose logs -f`
6. **Build issues**: Use `./run.sh build` for clean Docker builds

## Testing Guidelines

### Test Structure

The project follows standard Rust testing conventions:

#### Individual Crate Tests
- **`mcp-common/tests/`** - Unit and integration tests for common library (73 tests)
  - `types_tests.rs` - Data structure validation
  - `messages_tests.rs` - IPC message serialization
  - `mcp_tests.rs` - JSON-RPC protocol handling
  - `ipc_tests.rs` - IPC client/server functionality
  - `ipc_integration_tests.rs` - Full IPC communication scenarios
  
- **`mcp-proxy/tests/`** - Unit tests for proxy functionality (13 tests)
  - `buffered_ipc_client_tests.rs` - Buffered client with reconnection
  - `stdio_handler_tests.rs` - STDIO communication handling
  
- **`mcp-monitor/tests/`** - Unit tests for monitor app logic (16 tests)
  - `app_tests.rs` - Application state management and UI logic

#### Workspace-Level E2E Tests
- **`tests/`** - End-to-end system integration tests
  - `e2e/full_system_tests.rs` - Complete proxy-monitor scenarios

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p mcp-common
cargo test -p mcp-proxy  
cargo test -p mcp-monitor

# Run only E2E tests
cargo test --test e2e_tests

# Run with output for debugging
cargo test -- --nocapture
```

### Adding New Tests

#### Unit Tests
Add tests to the appropriate crate's `tests/` directory:

```rust
// mcp-common/tests/new_feature_tests.rs
use mcp_common::*;

#[test]
fn test_new_feature() {
    let result = new_function();
    assert_eq!(result, expected_value);
}
```

#### Integration Tests
Add to existing integration test files or create new ones in `mcp-common/tests/`:

```rust
#[tokio::test]
async fn test_new_integration_scenario() {
    // Test multiple components working together
}
```

#### E2E Tests
Add to `tests/e2e/full_system_tests.rs` for complete system scenarios:

```rust
#[tokio::test]
async fn test_new_end_to_end_scenario() {
    // Test complete proxy + monitor interaction
}
```

### Test Coverage
- **Core functionality**: Types, serialization, protocol handling
- **IPC communication**: Client/server, reconnection, error handling
- **Application logic**: State management, filtering, search
- **System integration**: Full proxy-monitor workflows

## Future Enhancement Areas

### WebSocket Support
- Add WebSocket proxy capability alongside STDIO
- Implement bidirectional message forwarding
- Add WebSocket-specific logging and statistics

### Advanced Filtering
- Implement log filtering by proxy, level, or content
- Add search functionality in the TUI
- Support regex patterns for log filtering

### Configuration Management
- Add configuration file support (TOML/YAML)
- Implement proxy profiles and saved configurations
- Add environment-based configuration

### Enhanced Statistics
- Add historical statistics tracking
- Implement performance metrics (latency, throughput)
- Add statistics export functionality

### Testing Infrastructure
- ✅ Unit tests for core functionality (102 tests across all crates)
- ✅ Integration tests with mock MCP servers  
- ✅ End-to-end tests for full system scenarios
- Add property-based testing for IPC protocol
