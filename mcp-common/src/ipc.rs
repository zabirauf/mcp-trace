use crate::{IpcEnvelope, IpcMessage};
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info};

pub struct IpcServer {
    listener: UnixListener,
}

impl IpcServer {
    pub async fn bind(socket_path: &str) -> Result<Self> {
        // Remove existing socket file if it exists
        let _ = tokio::fs::remove_file(socket_path).await;

        let listener = UnixListener::bind(socket_path)?;
        info!("IPC server listening on {}", socket_path);

        Ok(Self { listener })
    }

    pub async fn accept(&self) -> Result<IpcConnection> {
        let (stream, _) = self.listener.accept().await?;
        Ok(IpcConnection::new(stream))
    }
}

pub struct IpcConnection {
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: tokio::net::unix::OwnedWriteHalf,
}

impl IpcConnection {
    pub fn new(stream: UnixStream) -> Self {
        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::new(read_half);

        Self {
            reader,
            writer: write_half,
        }
    }

    pub async fn connect(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self::new(stream))
    }

    pub async fn send_message(&mut self, message: IpcMessage) -> Result<()> {
        let envelope = IpcEnvelope {
            message,
            timestamp: chrono::Utc::now(),
            correlation_id: Some(uuid::Uuid::new_v4()),
        };

        let json = serde_json::to_string(&envelope)?;
        debug!("Sending IPC message: {}", json);

        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;

        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<Option<IpcEnvelope>> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Ok(None); // Connection closed
        }

        match serde_json::from_str::<IpcEnvelope>(&line.trim()) {
            Ok(envelope) => {
                debug!("Received IPC message: {:?}", envelope.message);
                Ok(Some(envelope))
            }
            Err(e) => {
                error!("Failed to deserialize IPC message: {}", e);
                Err(e.into())
            }
        }
    }
}

pub struct IpcClient {
    connection: IpcConnection,
}

impl IpcClient {
    pub async fn connect(socket_path: &str) -> Result<Self> {
        let connection = IpcConnection::connect(socket_path).await?;
        Ok(Self { connection })
    }

    pub async fn send(&mut self, message: IpcMessage) -> Result<()> {
        self.connection.send_message(message).await
    }

    pub async fn receive(&mut self) -> Result<Option<IpcEnvelope>> {
        self.connection.receive_message().await
    }
}
