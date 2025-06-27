use anyhow::Result;
use mcp_common::{IpcClient, IpcMessage};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, error, info, warn};

const MAX_BUFFER_SIZE: usize = 10_000; // Maximum number of messages to buffer
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(30);
const RECONNECT_BACKOFF_FACTOR: u32 = 2;

pub struct BufferedIpcClient {
    buffer: Arc<Mutex<VecDeque<IpcMessage>>>,
    sender: mpsc::Sender<IpcMessage>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl BufferedIpcClient {
    pub async fn new(socket_path: String) -> Self {
        let buffer = Arc::new(Mutex::new(VecDeque::new()));
        let (sender, receiver) = mpsc::channel(1000);
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        // Start the background task
        let task_handle = tokio::spawn(Self::run_client_task(
            socket_path,
            buffer.clone(),
            receiver,
            shutdown_rx,
        ));

        let client = Self {
            buffer,
            sender,
            shutdown_tx: Some(shutdown_tx),
            task_handle: Some(task_handle),
        };

        client
    }

    pub async fn send(&self, message: IpcMessage) -> Result<()> {
        // Try to send through the channel (which will handle buffering if needed)
        if let Err(_) = self.sender.send(message.clone()).await {
            // If channel is full or closed, add directly to buffer
            let mut buffer = self.buffer.lock().await;
            if buffer.len() < MAX_BUFFER_SIZE {
                buffer.push_back(message);
            } else {
                warn!("Buffer full, dropping message");
            }
        }
        Ok(())
    }

    async fn run_client_task(
        socket_path: String,
        buffer: Arc<Mutex<VecDeque<IpcMessage>>>,
        mut receiver: mpsc::Receiver<IpcMessage>,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) {
        let mut client: Option<IpcClient> = None;
        let mut reconnect_delay = INITIAL_RECONNECT_DELAY;
        let mut last_connect_attempt = Instant::now() - reconnect_delay;

        loop {
            tokio::select! {
                // Check for shutdown
                _ = shutdown_rx.recv() => {
                    info!("BufferedIpcClient shutting down");
                    break;
                }

                // Try to receive new messages
                Some(message) = receiver.recv() => {
                    // Try to send the message
                    if let Some(ref mut ipc_client) = client {
                        if let Err(e) = ipc_client.send(message.clone()).await {
                            warn!("Failed to send message, will buffer: {}", e);
                            // Connection failed, reset client
                            client = None;
                            // Buffer the message
                            let mut buf = buffer.lock().await;
                            if buf.len() < MAX_BUFFER_SIZE {
                                buf.push_back(message);
                            }
                        }
                    } else {
                        // No connection, buffer the message
                        let mut buf = buffer.lock().await;
                        if buf.len() < MAX_BUFFER_SIZE {
                            buf.push_back(message);
                        }
                    }
                }

                // Periodic reconnection attempts
                _ = sleep(Duration::from_millis(100)) => {
                    if client.is_none() && last_connect_attempt.elapsed() >= reconnect_delay {
                        last_connect_attempt = Instant::now();

                        match IpcClient::connect(&socket_path).await {
                            Ok(new_client) => {
                                info!("Successfully connected to monitor at {}", socket_path);
                                client = Some(new_client);
                                reconnect_delay = INITIAL_RECONNECT_DELAY;

                                // Flush buffered messages
                                let messages_to_send: Vec<IpcMessage> = {
                                    let mut buf = buffer.lock().await;
                                    buf.drain(..).collect()
                                };

                                if !messages_to_send.is_empty() {
                                    info!("Flushing {} buffered messages", messages_to_send.len());
                                    if let Some(ref mut ipc_client) = client {
                                        for msg in messages_to_send {
                                            if let Err(e) = ipc_client.send(msg.clone()).await {
                                                error!("Failed to flush buffered message: {}", e);
                                                // Re-buffer failed messages
                                                let mut buf = buffer.lock().await;
                                                if buf.len() < MAX_BUFFER_SIZE {
                                                    buf.push_back(msg);
                                                }
                                                // Connection failed during flush
                                                client = None;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                debug!("Failed to connect to monitor (will retry): {}", e);
                                // Exponential backoff
                                reconnect_delay = std::cmp::min(
                                    reconnect_delay * RECONNECT_BACKOFF_FACTOR,
                                    MAX_RECONNECT_DELAY
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn shutdown(mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }

        // Wait for the background task to complete
        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }
    }
}

impl Drop for BufferedIpcClient {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.try_send(());
        }

        // Abort the background task to ensure test cleanup
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}
