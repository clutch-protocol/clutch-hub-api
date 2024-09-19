// src/hub/websocket_manager/connection.rs

use super::types::JSONRPCResponse;
use super::utils::extract_id_from_response;
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, info};

pub async fn start_connection_loop(
    url: String,
    mut rx: mpsc::Receiver<String>,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
) {
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
                                match serde_json::from_str::<JSONRPCResponse>(&text) {
                                    Ok(response) => {
                                        let mut pending = pending_requests.lock().await;
                                        if let Some(resp_tx) = pending.remove(&response.id) {
                                            let _ = resp_tx.send(text);
                                        } else {
                                            // Handle unexpected responses or notifications
                                            info!("Received unexpected message: {}", text);
                                        }
                                    },
                                    Err(e) => {
                                        error!("Failed to parse response: {}. Error: {}", text, e);
                                        // Remove the pending request to prevent it from hanging
                                        if let Some(id) = extract_id_from_response(&text) {
                                            let mut pending = pending_requests.lock().await;
                                            pending.remove(&id);
                                        }
                                    }
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

                // Notify pending requests about the disconnection
                let mut pending = pending_requests.lock().await;
                for (_, sender) in pending.drain() {
                    let _ = sender.send("".to_string());
                }
            }
            Err(e) => {
                error!("Failed to connect to WebSocket server at {}: {}", url, e);
            }
        }

        // Wait before attempting to reconnect
        let retry_seconds = 5;
        info!("Reconnecting in {} seconds...", retry_seconds);
        tokio::time::sleep(Duration::from_secs(retry_seconds)).await;
    }
}
