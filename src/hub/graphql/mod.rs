pub mod mutation;
pub mod query;
pub mod types;
pub mod handler;  
use std::sync::Arc;

pub use mutation::Mutation;
pub use query::Query;

use async_graphql::{Schema, EmptySubscription};

use super::websocket_manager::WebSocketManager;

// Function to build the GraphQL schema
pub fn build_schema(ws_manager: Arc<WebSocketManager>) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
    .data(ws_manager) 
    .finish()
}