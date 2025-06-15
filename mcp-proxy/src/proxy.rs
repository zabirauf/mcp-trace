use anyhow::Result;
use mcp_common::{
    IpcClient, IpcMessage, JsonRpcMessage, LogEntry, LogLevel, ProxyId, ProxyInfo, ProxyStats, ProxyStatus
};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

use crate::stdio_handler::StdioHandler;

pub struct MCPProxy {
    id: ProxyId,
    name: String,
    command: String,
    use_shell: bool,
    stats: Arc<Mutex<ProxyStats>>,
    start_time: Instant,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl MCPProxy {
    pub async fn new(id: ProxyId, name: String, command: String, use_shell: bool) -> Result<Self> {
        let mut stats = ProxyStats::default();
        stats.proxy_id = id.clone();
        
        Ok(Self {
            id,
            name,
            command,
            use_shell,
            stats: Arc::new(Mutex::new(stats)),
            start_time: Instant::now(),
            shutdown_tx: None,
        })
    }

    pub async fn start(&mut self, ipc_socket_path: Option<&str>) -> Result<()> {
        info!("Starting MCP proxy: {}", self.name);
        
        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Connect to monitor (optional)
        let mut ipc_client = if let Some(socket_path) = ipc_socket_path {
            match IpcClient::connect(socket_path).await {
                Ok(client) => {
                    info!("Connected to monitor at {}", socket_path);
                    Some(client)
                }
                Err(_) => {
                    debug!("Monitor not available. Running in standalone mode.");
                    None
                }
            }
        } else {
            info!("Running in standalone mode (monitor disabled)");
            None
        };

        // Send proxy started message
        if let Some(ref mut client) = ipc_client {
            let proxy_info = ProxyInfo {
                id: self.id.clone(),
                name: self.name.clone(),
                listen_address: "stdio".to_string(),
                target_command: vec![self.command.clone()],
                status: ProxyStatus::Starting,
                stats: self.stats.lock().await.clone(),
            };
            
            if let Err(e) = client.send(IpcMessage::ProxyStarted(proxy_info)).await {
                warn!("Failed to send proxy started message: {}", e);
            }
        }

        // Start MCP server process
        let mut child = self.start_mcp_server().await?;
        
        // Create STDIO handler
        let mut handler = StdioHandler::new(
            self.id.clone(),
            self.stats.clone(),
            ipc_client,
        ).await?;

        // Update status to running
        {
            let mut stats = self.stats.lock().await;
            // Note: ProxyStats doesn't have a status field, but we track it in ProxyInfo
        }

        // Handle STDIO communication
        let result = handler.handle_communication(&mut child, shutdown_rx).await;
        
        // Clean up
        info!("Proxy {} shutting down", self.name);
        if let Err(e) = child.kill().await {
            warn!("Failed to kill MCP server process: {}", e);
        }

        // Send proxy stopped message
        if let Some(mut client) = handler.take_ipc_client() {
            if let Err(e) = client.send(IpcMessage::ProxyStopped(self.id.clone())).await {
                warn!("Failed to send proxy stopped message: {}", e);
            }
        }

        result
    }

    async fn start_mcp_server(&self) -> Result<Child> {
        if self.command.is_empty() {
            return Err(anyhow::anyhow!("No command specified"));
        }

        let child = if self.use_shell {
            // Use shell to execute the command
            Command::new("sh")
                .arg("-c")
                .arg(&self.command)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        } else {
            // Parse command and arguments
            let parts: Vec<&str> = self.command.split_whitespace().collect();
            if parts.is_empty() {
                return Err(anyhow::anyhow!("Empty command"));
            }
            
            let mut cmd = Command::new(parts[0]);
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }
            
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        };

        info!("Started MCP server process: {}", self.command);
        Ok(child)
    }

    pub async fn get_stats(&self) -> ProxyStats {
        let mut stats = self.stats.lock().await.clone();
        stats.uptime = self.start_time.elapsed();
        stats
    }
}