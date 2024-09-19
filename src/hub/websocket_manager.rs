use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, info};

pub struct WebSocketManager {
    tx: mpsc::Sender<String>,
}

impl WebSocketManager {
    /// Creates a new WebSocketManager and starts the connection task.
    pub fn new(url: String) -> Self {
        let (tx, rx) = mpsc::channel(100);

        // Start the background connection task
        let url_clone = url.clone();
        tokio::spawn(async move {
            start_connection_loop(url_clone, rx).await;
        });

        WebSocketManager { tx }
    }

    /// Sends a message to the WebSocket server.
    pub async fn send_message(&self, message: &str) {
        if let Err(e) = self.tx.send(message.to_string()).await {
            error!("Failed to send message to queue: {}", e);
        } else {
            info!("Message queued: {}", message);
        }
    }
}

async fn start_connection_loop(url: String, mut rx: mpsc::Receiver<String>) {
    loop {
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                info!("Connected to WebSocket server at {}", url);
                let (mut ws_sink, mut ws_stream) = ws_stream.split();

                let mut connected = true;

                while connected {
                    tokio::select! {
                        // Handle outgoing messages
                        Some(message) = rx.recv() => {
                            if let Err(e) = ws_sink.send(Message::Text(message)).await {
                                error!("Failed to send message: {}", e);
                                connected = false;
                            }
                        }
                        // Handle incoming messages from the server
                        Some(Ok(msg)) = ws_stream.next() => {
                            // Process incoming messages if needed
                            info!("Received message: {:?}", msg);
                        }
                        // Handle connection closure
                        else => {
                            error!("WebSocket connection closed");
                            connected = false;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to WebSocket server at {}: {}", url, e);
            }
        }

        // Wait before attempting to reconnect
        let retry_seconds = 5;
        info!("Reconnecting in {} seconds...", retry_seconds);
        tokio::time::sleep(tokio::time::Duration::from_secs(retry_seconds)).await;
    }
}
