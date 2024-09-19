use super::connection::start_connection_loop;
use super::types::{JSONRPCRequest, JSONRPCResponse};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::{timeout, Duration};
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
        let pending_requests_clone = pending_requests.clone();
        tokio::spawn(async move {
            start_connection_loop(url, rx, pending_requests_clone).await;
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

        // Send the request
        if let Err(e) = self.tx.send(request_json).await {
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
    }
}
