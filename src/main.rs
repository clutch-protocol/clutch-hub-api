use actix_web::{web, App, HttpServer};
use crate::graphql::handler::graphql_handler;

mod graphql;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = graphql::build_schema();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/graphql").route(web::post().to(graphql_handler)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
