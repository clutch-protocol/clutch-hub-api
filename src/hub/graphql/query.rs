use std::sync::Arc;

use crate::hub::{graphql::types::RideRequest, websocket_manager::WebSocketManager};
use async_graphql::{Context, Object};
use serde_json::json;
use tracing::error;

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    pub async fn ride_request(&self, ctx: &Context<'_>, user_id: String) -> Option<RideRequest> {
        let ws_manager = ctx
            .data::<Arc<WebSocketManager>>()
            .expect("WebSocketManager not found in context");
    
        let params = json!({ "user_id": user_id });
    
        match ws_manager.send_request("ride_request", params).await {
            Ok(result) => {
                // Parse the result into RideRequest
                match serde_json::from_value::<RideRequest>(result) {
                    Ok(ride_request) => Some(ride_request),
                    Err(e) => {
                        error!("Failed to parse RideRequest: {}", e);
                        None
                    }
                }
            },
            Err(e) => {
                error!("Failed to send request: {}", e);
                // Handle the error, e.g., return None or propagate the error
                None
            }
        }
    }
}
