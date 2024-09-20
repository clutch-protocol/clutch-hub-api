use crate::hub::graphql::types::RideRequest;
use async_graphql::{Context, Object};

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    pub async fn ride_request(&self, _ctx: &Context<'_>, user_id: String) -> Option<RideRequest> {
        Some(RideRequest {
            pickup_location: "0".to_string(),
            dropoff_location: "0".to_string(),
            user_id: user_id,
        })
    }
}
