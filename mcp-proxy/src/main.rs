use anyhow::Result;
use clap::Parser;
use mcp_proxy::{run_proxy_app, ProxyArgs};

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
    
    let proxy_args = ProxyArgs {
        command: args.command,
        name: args.name,
        ipc_socket: args.ipc_socket,
        verbose: args.verbose,
        shell: args.shell,
        no_monitor: args.no_monitor,
    };
    
    run_proxy_app(proxy_args).await
}