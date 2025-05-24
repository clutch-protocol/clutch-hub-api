use super::connection::start_connection_loop;
use super::types::{JSONRPCRequest, JSONRPCResponse};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use serde_json;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{oneshot, Mutex};
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{error, info};
use uuid::Uuid;

pub struct ClutchNodeClient {
    ws_sink: Arc<Mutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
}

impl ClutchNodeClient {
    /// Creates a new WebSocketManager and starts the connection task.
    pub fn new(url: String) -> Arc<Self> {
        let ws_sink = Arc::new(Mutex::new(None));
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));

        // Start the background connection task
        let ws_sink_clone = ws_sink.clone();
        let pending_requests_clone = pending_requests.clone();
        tokio::spawn(async move {
            start_connection_loop(url, ws_sink_clone, pending_requests_clone).await;
        });

        Arc::new(ClutchNodeClient {
            ws_sink,
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
        
        // Format the request based on the method type
        // For send_raw_transaction, params should be a direct string not an object
        let request = if method == "send_raw_transaction" {
            // For send_raw_transaction, extract the string from the Value
            let tx_string = match &params {
                serde_json::Value::String(s) => s.clone(),
                _ => params.as_str().unwrap_or_default().to_string(),
            };
            
            JSONRPCRequest {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params: serde_json::Value::String(tx_string),
                id: id.clone(),
            }
        } else {
            // For other methods, use the params as provided
            JSONRPCRequest {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params,
                id: id.clone(),
            }
        };

        let request_json = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        
        // Log the actual request being sent for debugging
        info!("Sending request to node: {}", request_json);

        // Check if the connection is established
        let mut ws_sink_lock = self.ws_sink.lock().await;
        if let Some(ws_sink) = ws_sink_lock.as_mut() {
            let (resp_tx, resp_rx) = oneshot::channel();

            {
                let mut pending = self.pending_requests.lock().await;
                pending.insert(id.clone(), resp_tx);
            }

            // Send the request
            if let Err(e) = ws_sink.send(Message::Text(request_json)).await {
                // Sending failed, remove the pending request
                let mut pending = self.pending_requests.lock().await;
                pending.remove(&id);
                return Err(format!("Failed to send request: {}", e));
            }

            // Wait for response with timeout
            let response_result = timeout(Duration::from_secs(10), resp_rx).await;

            match response_result {
                Ok(Ok(response_json)) => {
                    if response_json.is_empty() {
                        // Connection lost, and no response received
                        return Err("Connection lost before receiving response".to_string());
                    }

                    // Log the response for debugging
                    info!("Received response: {}", response_json);

                    // Parse the response
                    let response: JSONRPCResponse =
                        serde_json::from_str(&response_json).map_err(|e| e.to_string())?;
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
                }
                Ok(Err(_)) => {
                    // Sender was dropped
                    Err("Failed to receive response".to_string())
                }
                Err(_) => {
                    // Timeout occurred
                    let mut pending = self.pending_requests.lock().await;
                    pending.remove(&id);
                    Err("Request timed out".to_string())
                }
            }
        } else {
            Err("WebSocket connection not established".to_string())
        }
    }

    /// Gets the next nonce value for the given address.
    pub async fn get_next_nonce(&self, address: &str) -> u64 {
        // Request the next nonce from the node
        match self.send_request("get_next_nonce", json!({ "address": address })).await {
            Ok(result) => {
                // Try to parse the nonce value from the result
                match result.get("nonce").and_then(|n| n.as_u64()) {
                    Some(nonce) => {
                        info!("Retrieved nonce {} for address {}", nonce, address);
                        nonce
                    },
                    None => {
                        error!("Failed to parse nonce value from response: {:?}", result);
                        // Fallback to default nonce if parsing fails
                        1
                    }
                }
            },
            Err(e) => {
                error!("Failed to get nonce for address {}: {}", address, e);
                // Fallback to default nonce if request fails
                1
            }
        }
    }
}
