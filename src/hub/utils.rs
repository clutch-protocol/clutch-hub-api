// src/hub/websocket_manager/utils.rs

use serde_json;

pub fn extract_id_from_response(response: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(response) {
        value
            .get("id")
            .and_then(|id| id.as_str().map(|s| s.to_string()))
    } else {
        None
    }
}
