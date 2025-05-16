use crate::hub::clutch_node_client::ClutchNodeClient;
use crate::hub::configuration::AppConfig;
use crate::hub::graphql::build_schema;
use crate::hub::graphql::handler::graphql_handler;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use actix_cors::Cors;

pub async fn connect_websocket(wss_url: &str) -> Arc<ClutchNodeClient> {
    let url = wss_url.to_string();
    ClutchNodeClient::new(url)
}

pub async fn run_graphql_server(
    ws_addr: &str,
    ws_manager: Arc<ClutchNodeClient>,
    config: AppConfig,
) -> std::io::Result<()> {
    let schema = build_schema(ws_manager.clone(), config.clone());
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["POST", "OPTIONS"])
                    .allow_any_header()
            )
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/graphql").route(web::post().to(graphql_handler)))
    })
    .bind(ws_addr)?
    .run()
    .await
}
