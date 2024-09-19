use async_graphql::{Context, Object};
use crate::hub::graphql::types::RideRequest;

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    pub async fn ride_request(&self, _ctx: &Context<'_>, user_id: String) -> Option<RideRequest> {
       
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
