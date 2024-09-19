use std::sync::Arc;

use crate::hub::{graphql::types::RideRequest, websocket_manager::WebSocketManager};
use async_graphql::{Context, Object};
use tracing::{error, info};

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    pub async fn ride_request(&self, ctx: &Context<'_>, user_id: String) -> Option<RideRequest> {
        let ws_manager = ctx
            .data::<Arc<WebSocketManager>>()
            .expect("WebSocketManager not found in context");

        if let Err(e) = ws_manager.send_message("Your message here").await {
            error!("Failed to send message: {}", e);
        }

        // Optionally, receive messages
        if let Some(response) = ws_manager.receive_message().await {
            info!("Received response: {}", response);
        }

        Some(RideRequest {
            pickup_location: "Pickup".to_string(),
            dropoff_location: "Dropoff".to_string(),
            user_id,
        })
    }

    pub async fn ride_offer(&self, _ctx: &Context<'_>, user_id: String) -> Option<RideRequest> {
        // Dummy data - replace this with database lookup or other logic
        Some(RideRequest {
            pickup_location: "Pickup".to_string(),
            dropoff_location: "Dropoff".to_string(),
            user_id,
        })
    }
}
