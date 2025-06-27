use anyhow::Result;
use mcp_common::ProxyId;
use tracing::info;

mod buffered_ipc_client;
mod proxy;
mod stdio_handler;

use proxy::MCPProxy;

// Export modules for testing
pub use buffered_ipc_client::BufferedIpcClient;
pub use stdio_handler::StdioHandler;

pub struct ProxyArgs {
    pub command: String,
    pub name: String,
    pub ipc_socket: String,
    pub verbose: bool,
    pub shell: bool,
    pub no_monitor: bool,
}

pub async fn run_proxy_app(args: ProxyArgs) -> Result<()> {
    // Initialize tracing
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("mcp_proxy={},mcp_common={}", log_level, log_level))
        .init();

    info!("Starting MCP Proxy: {}", args.name);
    info!("Target command: {}", args.command);

    if args.command.is_empty() {
        return Err(anyhow::anyhow!(
            "No command specified. Use --command to specify the MCP server command."
        ));
    }

    // Create proxy instance
    let proxy_id = ProxyId::new();
    let mut proxy = MCPProxy::new(
        proxy_id.clone(),
        args.name.clone(),
        args.command.clone(),
        args.shell,
    )
    .await?;

    // Start the proxy
    let ipc_socket = if args.no_monitor {
        None
    } else {
        Some(args.ipc_socket.as_str())
    };
    proxy.start(ipc_socket).await?;

    Ok(())
}
