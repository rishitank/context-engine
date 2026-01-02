//! MCP transport layer implementations.
//!
//! Supports stdio and HTTP/SSE transports.

use async_trait::async_trait;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, error, trace};

use crate::error::Result;
use crate::mcp::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// A message that can be sent or received.
#[derive(Debug, Clone)]
pub enum Message {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

/// Transport trait for MCP communication.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Start the transport, returning channels for messages.
    async fn start(&mut self) -> Result<(
        mpsc::Receiver<Message>,
        mpsc::Sender<Message>,
    )>;

    /// Stop the transport.
    async fn stop(&mut self) -> Result<()>;
}

/// Stdio transport for MCP.
pub struct StdioTransport {
    running: bool,
}

impl StdioTransport {
    /// Create a new stdio transport.
    pub fn new() -> Self {
        Self { running: false }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn start(&mut self) -> Result<(mpsc::Receiver<Message>, mpsc::Sender<Message>)> {
        self.running = true;

        // Channel for incoming messages (from stdin)
        let (incoming_tx, incoming_rx) = mpsc::channel::<Message>(100);
        // Channel for outgoing messages (to stdout)
        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<Message>(100);

        // Spawn stdin reader task
        let tx = incoming_tx.clone();
        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut reader = BufReader::new(stdin);
            let mut line = String::new();

            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        debug!("EOF on stdin, stopping transport");
                        break;
                    }
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        trace!("Received: {}", trimmed);

                        // Try to parse as request first, then notification
                        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(trimmed) {
                            if tx.send(Message::Request(req)).await.is_err() {
                                break;
                            }
                        } else if let Ok(notif) = serde_json::from_str::<JsonRpcNotification>(trimmed) {
                            if tx.send(Message::Notification(notif)).await.is_err() {
                                break;
                            }
                        } else {
                            error!("Failed to parse message: {}", trimmed);
                        }
                    }
                    Err(e) => {
                        error!("Error reading stdin: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn stdout writer task
        tokio::spawn(async move {
            let mut stdout = tokio::io::stdout();

            while let Some(msg) = outgoing_rx.recv().await {
                let json = match &msg {
                    Message::Request(req) => serde_json::to_string(req),
                    Message::Response(res) => serde_json::to_string(res),
                    Message::Notification(notif) => serde_json::to_string(notif),
                };

                match json {
                    Ok(s) => {
                        trace!("Sending: {}", s);
                        if let Err(e) = stdout.write_all(s.as_bytes()).await {
                            error!("Error writing to stdout: {}", e);
                            break;
                        }
                        if let Err(e) = stdout.write_all(b"\n").await {
                            error!("Error writing newline: {}", e);
                            break;
                        }
                        if let Err(e) = stdout.flush().await {
                            error!("Error flushing stdout: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error serializing message: {}", e);
                    }
                }
            }
        });

        Ok((incoming_rx, outgoing_tx))
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }
}

