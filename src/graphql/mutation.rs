use async_graphql::{Context, Object};
use crate::graphql::types::RideRequest;

#[derive(Default)]
pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn create_ride_request(
        &self,
        _ctx: &Context<'_>,
        pickup_location: String,
        dropoff_location: String,
        user_id: String,
    ) -> RideRequest {
        // Here you'd add logic to store this request in a database or other storage.
        RideRequest {
            pickup_location,
            dropoff_location,
            user_id,
        }
    }
}
