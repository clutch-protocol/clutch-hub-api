use actix_web::{web, App, HttpServer};
use async_graphql::{EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

mod graphql;

async fn graphql_handler(
    schema: web::Data<Schema<graphql::Query, graphql::Mutation, EmptySubscription>>,
    req: GraphQLRequest
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::build(graphql::Query::default(), graphql::Mutation::default(), EmptySubscription)
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
