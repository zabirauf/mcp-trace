use anyhow::Result;
use clap::Parser;
use mcp_common::{IpcClient, IpcMessage, LogEntry, LogLevel, ProxyId, ProxyInfo, ProxyStats, ProxyStatus};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::time::{interval, Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

mod proxy;
mod stdio_handler;

use proxy::MCPProxy;

#[derive(Parser)]
#[command(name = "mcp-proxy")]
#[command(about = "STDIO-based MCP proxy server")]
pub struct Args {
    /// MCP server command to proxy (as a single string, will be executed via shell)
    #[arg(short, long)]
    pub command: String,
    
    /// Name for this proxy instance
    #[arg(short, long, default_value = "mcp-proxy")]
    pub name: String,
    
    /// IPC socket path for monitor communication
    #[arg(short, long, default_value = "/tmp/mcp-monitor.sock")]
    pub ipc_socket: String,
    
    /// Verbose logging
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Use shell to execute command (enabled by default)
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub shell: bool,
    
    /// Skip connecting to monitor (standalone mode)
    #[arg(long, default_value_t = false)]
    pub no_monitor: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize tracing
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("mcp_proxy={},mcp_common={}", log_level, log_level))
        .init();

    info!("Starting MCP Proxy: {}", args.name);
    info!("Target command: {}", args.command);
    
    if args.command.is_empty() {
        return Err(anyhow::anyhow!("No command specified. Use --command to specify the MCP server command."));
    }

    // Create proxy instance
    let proxy_id = ProxyId::new();
    let mut proxy = MCPProxy::new(proxy_id.clone(), args.name.clone(), args.command.clone(), args.shell).await?;
    
    // Start the proxy
    let ipc_socket = if args.no_monitor { None } else { Some(args.ipc_socket.as_str()) };
    proxy.start(ipc_socket).await?;
    
    Ok(())
}