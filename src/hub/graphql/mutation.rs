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
use thiserror::Error;

// Custom error type for mutations
#[derive(Debug, Error)]
pub enum MutationError {
    #[error("Authentication failed: {0}")]
    AuthError(String),
    #[error("Internal server error: {0}")]
    InternalError(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

#[derive(Default)]
pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn generate_token(
        &self,
        ctx: &Context<'_>,
        public_key: String,
    ) -> async_graphql::Result<TokenResponse> {
        let config = ctx
            .data::<AppConfig>()
            .map_err(|_| async_graphql::Error::new("Failed to get app config"))?;

        let (token, expires_at) = auth::generate_jwt_token(
            &public_key,
            config.jwt_expiration_hours,
            config.jwt_secret.as_str(),
        )
        .map_err(|e| async_graphql::Error::new(format!("Failed to generate token: {}", e)))?;

        Ok(TokenResponse { token, expires_at })
    }

    #[graphql(guard = "AuthGuard")]
    pub async fn create_unsigned_ride_request(  
        &self,
        ctx: &Context<'_>,
        pickup_latitude: f64,
        pickup_longitude: f64,
        dropoff_latitude: f64,
        dropoff_longitude: f64,
        fare: i32,
    ) -> async_graphql::Result<RideRequest> {
        // Get authenticated user from context
        let auth_user = get_auth_user(ctx)
            .ok_or_else(|| async_graphql::Error::new("User not authenticated"))?;

        info!(
            "Processing ride request for user with public key: {}",
            auth_user.public_key
        );

        let client = ctx
            .data::<Arc<ClutchNodeClient>>()
            .map_err(|_| async_graphql::Error::new("WebSocket manager not found"))?
            .clone();

        // Get the next nonce for this user using the client method
        let nonce = client.get_next_nonce(&auth_user.public_key).await;

        // Create request parameters
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

        // Send request and handle response
        let result = client
            .send_request("send_transaction", params)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to send request: {}", e)))?;

        // Parse the result into RideRequest
        serde_json::from_value::<RideRequest>(result)
            .map_err(|e| async_graphql::Error::new(format!("Failed to parse RideRequest: {}", e)))
    }
}
