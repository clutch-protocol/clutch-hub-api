use std::sync::Arc;

use async_graphql::{Context, Object};
use serde_json::json;
use tracing::error;
use crate::hub::{clutch_node_client::ClutchNodeClient, graphql::types::RideRequest};

#[derive(Default)]
pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn create_ride_request(
        &self,
        ctx: &Context<'_>,
        _pickup_location: String,
        _dropoff_location: String,
        user_id: String,
    ) -> Option<RideRequest> {   
        let ws_manager = ctx
        .data::<Arc<ClutchNodeClient>>()
        .expect("WebSocketManager not found in context");

    let params = json!({ "user_id": user_id });

    match ws_manager.send_request("send_transaction", params).await {
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
