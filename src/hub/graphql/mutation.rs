use std::sync::Arc;

use crate::hub::{
    auth,
    clutch_node_client::ClutchNodeClient,
    configuration::AppConfig,
    graphql::types::{get_auth_user, AuthGuard, RideRequest, TokenResponse},
};
use async_graphql::{Context, Object};
use serde_json::json;
use tracing::{error, info};

#[derive(Default)]
pub struct Mutation;

// Function to get the next nonce value
async fn get_next_nonce(client: &ClutchNodeClient, address: &str) -> u64 {
    // Request the next nonce from the node
    match client.send_request("get_next_nonce", json!({ "address": address })).await {
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

#[Object]
impl Mutation {
    pub async fn generate_token(
        &self,
        ctx: &Context<'_>,
        public_key: String,
    ) -> async_graphql::Result<TokenResponse> {
        let config = ctx
            .data::<AppConfig>()
            .map_err(|_| async_graphql::Error::new("Internal server error"))?;

        let (token, expires_at) = auth::generate_jwt_token(
            &public_key,
            config.jwt_expiration_hours,
            config.jwt_secret.as_str(),
        )
        .map_err(|e| async_graphql::Error::new(format!("Failed to generate token: {}", e)))?;

        Ok(TokenResponse { token, expires_at })
    }

    #[graphql(guard = "AuthGuard")]
    pub async fn create_ride_request(
        &self,
        ctx: &Context<'_>,
        pickup_latitude: f64,
        pickup_longitude: f64,
        dropoff_latitude: f64,
        dropoff_longitude: f64,
        fare: i32,
    ) -> Option<RideRequest> {
        // Get authenticated user from context - we can safely unwrap because the guard ensures it exists
        let auth_user = get_auth_user(ctx).expect("User should be authenticated due to AuthGuard");

        info!(
            "Processing ride request for user with public key: {}",
            auth_user.public_key
        );

        let ws_manager = ctx
            .data::<Arc<ClutchNodeClient>>()
            .expect("WebSocketManager not found in context");

        // Get the next nonce for this user
        let nonce = get_next_nonce(&ws_manager, &auth_user.public_key).await;

        // Use the authenticated user's data in the request with the specified format
        let params = json!({
            "from": auth_user.public_key,
            "nonce": nonce,
            "data": {
                "function_call_type": "RideRequest",
                "arguments": {
                    "fare": fare,
                    "pickup_location": {
                        "latitude": pickup_latitude,
                        "longitude": pickup_longitude
                    },
                    "dropoff_location": {
                        "latitude": dropoff_latitude,
                        "longitude": dropoff_longitude
                    }
                }
            }
        });

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
            }
            Err(e) => {
                error!("Failed to send request: {}", e);
                // Handle the error, e.g., return None or propagate the error
                None
            }
        }
    }
}
