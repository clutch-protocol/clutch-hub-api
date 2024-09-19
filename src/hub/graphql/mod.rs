pub mod mutation;
pub mod query;
pub mod types;
pub mod handler;  
pub use mutation::Mutation;
pub use query::Query;

use async_graphql::{Schema, EmptySubscription};

// Function to build the GraphQL schema
pub fn build_schema() -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription).finish()
}