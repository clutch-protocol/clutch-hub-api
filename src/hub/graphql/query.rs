use std::sync::Arc;

use async_graphql::{Context, Object};
use crate::hub::{graphql::types::RideRequest, websocket_manager::WebSocketManager};

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    pub async fn ride_request(&self, ctx: &Context<'_>, user_id: String) -> Option<RideRequest> {
        let ws_manager = ctx.data::<Arc<WebSocketManager>>().expect("WebSocketManager not found in context");
        ws_manager.send_message("Your message here").await;
        
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
