use async_graphql::{Object, Schema, Context, SimpleObject, EmptySubscription};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use actix_web::{web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(SimpleObject, Serialize, Deserialize)]
struct RideRequest {
    pickup_location: String,
    dropoff_location: String,
    user_id: String,
}

#[derive(Default)]
struct Query;

#[Object]
impl Query {
    async fn ride_request(&self, ctx: &Context<'_>, user_id: String) -> Option<RideRequest> {
        // Dummy data - replace this with database lookup or other logic
        Some(RideRequest {
            pickup_location: "Pickup".to_string(),
            dropoff_location: "Dropoff".to_string(),
            user_id,
        })
    }
}

#[derive(Default)]
struct Mutation;

#[Object]
impl Mutation {
    async fn create_ride_request(
        &self,
        ctx: &Context<'_>,
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

async fn graphql_handler(
    schema: web::Data<Schema<Query, Mutation, EmptySubscription>>,  // Include EmptySubscription here
    req: GraphQLRequest
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription) // Include EmptySubscription here
        .finish();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/graphql").route(web::post().to(graphql_handler)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
