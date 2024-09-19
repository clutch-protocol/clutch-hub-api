use actix_web::web;
use async_graphql::{Schema, EmptySubscription};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use crate::hub::graphql::{Query, Mutation};

pub async fn graphql_handler(
    schema: web::Data<Schema<Query, Mutation, EmptySubscription>>,
    req: GraphQLRequest
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
