use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, info};
use std::sync::Arc;

pub struct WebSocketManager {
    tx: mpsc::Sender<String>,
    incoming_rx: Mutex<mpsc::Receiver<String>>,
}

impl WebSocketManager {
    /// Creates a new WebSocketManager and starts the connection task.
    pub fn new(url: String) -> Arc<Self> {
        let (tx, rx) = mpsc::channel(100);
        let (incoming_tx, incoming_rx) = mpsc::channel(100);

        // Start the background connection task
        let url_clone = url.clone();
        tokio::spawn(async move {
            start_connection_loop(url_clone, rx, incoming_tx).await;
        });

        Arc::new(WebSocketManager {
            tx,
            incoming_rx: Mutex::new(incoming_rx),
        })
    }

    /// Sends a message to the WebSocket server.
    pub async fn send_message(&self, message: &str) -> Result<(), String> {
        self.tx.send(message.to_string()).await.map_err(|e| {
            let error_msg = format!("Failed to send message to queue: {}", e);
            error!("{}", error_msg);
            error_msg
        })
    }

    /// Receives incoming messages from the WebSocket server.
    pub async fn receive_message(&self) -> Option<String> {
        let mut rx = self.incoming_rx.lock().await;
        rx.recv().await
    }
}

async fn start_connection_loop(url: String, mut rx: mpsc::Receiver<String>, incoming_tx: mpsc::Sender<String>) {
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
                            if let Message::Text(text) = msg {
                                // Send the incoming message to the application
                                if let Err(e) = incoming_tx.send(text).await {
                                    error!("Failed to send incoming message to application: {}", e);
                                }
                            }
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
