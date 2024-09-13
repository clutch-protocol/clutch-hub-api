use actix_web::{web, App, HttpServer};
use crate::graphql::handler::graphql_handler;
use crate::graphql::build_schema;

// Function to configure and run the HTTP server
pub async fn run_server() -> std::io::Result<()> {
    let schema = build_schema();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/graphql").route(web::post().to(graphql_handler)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
