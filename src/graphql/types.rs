use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct RideRequest {
    pub pickup_location: String,
    pub dropoff_location: String,
    pub user_id: String,
}
