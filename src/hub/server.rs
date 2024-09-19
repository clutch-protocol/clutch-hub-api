use crate::hub::graphql::build_schema;
use crate::hub::graphql::handler::graphql_handler;
use crate::hub::clutch_node_client::ClutchNodeClient;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

pub async fn connect_websocket(wss_url: &str) -> Arc<ClutchNodeClient> {
    let url = wss_url.to_string();
    ClutchNodeClient::new(url)
}

pub async fn run_graphql_server(ws_manager: Arc<ClutchNodeClient>) -> std::io::Result<()> {
    let schema = build_schema(ws_manager.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/graphql").route(web::post().to(graphql_handler)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
