use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, info};
use uuid::Uuid;

pub struct WebSocketManager {
    tx: mpsc::Sender<String>,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
}

impl WebSocketManager {
    /// Creates a new WebSocketManager and starts the connection task.
    pub fn new(url: String) -> Arc<Self> {
        let (tx, rx) = mpsc::channel(100);
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));

        // Start the background connection task
        let url_clone = url.clone();
        let pending_requests_clone = pending_requests.clone();
        tokio::spawn(async move {
            start_connection_loop(url_clone, rx, pending_requests_clone).await;
        });

        Arc::new(WebSocketManager {
            tx,
            pending_requests,
        })
    }

    /// Sends a request and awaits the response.
    pub async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let id = Uuid::new_v4().to_string();
        let (resp_tx, resp_rx) = oneshot::channel();

        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id.clone(), resp_tx);
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: id.clone(),
        };

        let request_json = serde_json::to_string(&request).map_err(|e| e.to_string())?;

        self.tx
            .send(request_json)
            .await
            .map_err(|e| e.to_string())?;

        // Wait for response
        match resp_rx.await {
            Ok(response_json) => {
                // Parse the response
                let response: JSONRPCResponse = serde_json::from_str(&response_json).map_err(|e| {
                    error!("Failed to parse response: {}. Error: {}", response_json, e);
                    e.to_string()
                })?;
                if response.id != id {
                    return Err("Mismatched response ID".to_string());
                }
                if let Some(error) = response.error {
                    Err(error.message)
                } else if let Some(result) = response.result {
                    Ok(result)
                } else {
                    Err("No result or error in response".to_string())
                }
            },
            Err(_) => Err("Failed to receive response".to_string()),
        }
    }
}

async fn start_connection_loop(
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

    fn extract_id_from_response(response: &str) -> Option<String> {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(response) {
            value.get("id").and_then(|id| id.as_str().map(|s| s.to_string()))
        } else {
            None
        }
    }
    
}

#[derive(Serialize, Deserialize)]
struct JSONRPCRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: String,
}

#[derive(Serialize, Deserialize)]
struct JSONRPCResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JSONRPCError>,
    id: String,
}

#[derive(Serialize, Deserialize)]
struct JSONRPCError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}
