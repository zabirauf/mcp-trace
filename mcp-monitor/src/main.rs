use anyhow::Result;
use clap::Parser;
use mcp_monitor::{run_monitor_app, MonitorArgs};

#[derive(Parser)]
#[command(name = "mcp-monitor")]
#[command(about = "Monitor for MCP proxy servers")]
pub struct Args {
    /// IPC socket path for proxy communication
    #[arg(short, long, default_value = "/tmp/mcp-monitor.sock")]
    pub ipc_socket: String,

    /// Verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let monitor_args = MonitorArgs {
        ipc_socket: args.ipc_socket,
        verbose: args.verbose,
    };

    run_monitor_app(monitor_args).await
}
