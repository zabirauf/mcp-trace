use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mcp-trace")]
#[command(about = "Unified MCP probing and monitoring tool")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the MCP monitor (default if no subcommand provided)
    Monitor {
        /// IPC socket path for proxy communication
        #[arg(short, long, default_value = "/tmp/mcp-monitor.sock")]
        ipc_socket: String,

        /// Verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    /// Start an MCP proxy server
    Proxy {
        /// MCP server command to proxy (as a single string, will be executed via shell)
        #[arg(short, long)]
        command: String,

        /// Name for this proxy instance
        #[arg(short, long, default_value = "mcp-proxy")]
        name: String,

        /// IPC socket path for monitor communication
        #[arg(short, long, default_value = "/tmp/mcp-monitor.sock")]
        ipc_socket: String,

        /// Verbose logging
        #[arg(short, long)]
        verbose: bool,

        /// Use shell to execute command (enabled by default)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        shell: bool,

        /// Skip connecting to monitor (standalone mode)
        #[arg(long, default_value_t = false)]
        no_monitor: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Monitor {
            ipc_socket,
            verbose,
        }) => run_monitor(ipc_socket, verbose).await,
        Some(Commands::Proxy {
            command,
            name,
            ipc_socket,
            verbose,
            shell,
            no_monitor,
        }) => run_proxy(command, name, ipc_socket, verbose, shell, no_monitor).await,
        None => {
            // Default to monitor
            run_monitor("/tmp/mcp-monitor.sock".to_string(), false).await
        }
    }
}

async fn run_monitor(ipc_socket: String, verbose: bool) -> Result<()> {
    // Import the monitor functionality
    use mcp_monitor::{run_monitor_app, MonitorArgs};

    let args = MonitorArgs {
        ipc_socket,
        verbose,
    };

    run_monitor_app(args).await
}

async fn run_proxy(
    command: String,
    name: String,
    ipc_socket: String,
    verbose: bool,
    shell: bool,
    no_monitor: bool,
) -> Result<()> {
    // Import the proxy functionality
    use mcp_proxy::{run_proxy_app, ProxyArgs};

    let args = ProxyArgs {
        command,
        name,
        ipc_socket,
        verbose,
        shell,
        no_monitor,
    };

    run_proxy_app(args).await
}
