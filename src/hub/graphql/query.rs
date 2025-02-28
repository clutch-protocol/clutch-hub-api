use crate::hub::graphql::types::{RideRequest, AuthGuard, get_auth_user};
use async_graphql::{Context, Object};

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    // This query requires authentication
    #[graphql(guard = "AuthGuard")]
    pub async fn user_ride_requests(&self, ctx: &Context<'_>) -> Option<RideRequest> {
        // Get authenticated user from context - safely unwrap because AuthGuard ensures it exists
        let auth_user = get_auth_user(ctx).expect("User should be authenticated due to AuthGuard");
        
        Some(RideRequest {
            pickup_location: "0".to_string(),
            dropoff_location: "0".to_string(),
        })
    }
    
    // This query doesn't require authentication
    pub async fn ride_request(&self, _ctx: &Context<'_>) -> Option<RideRequest> {
        Some(RideRequest {
            pickup_location: "0".to_string(),
            dropoff_location: "0".to_string(),
        })
    }
}
