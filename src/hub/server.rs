use crate::hub::graphql::build_schema;
use crate::hub::graphql::handler::graphql_handler;
use crate::hub::websocket_manager::WebSocketManager;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

pub async fn connect_websocket(wss_url:&str) -> Arc<WebSocketManager> {
    let url = wss_url.to_string();
    let ws_manager = Arc::new(WebSocketManager::new(
        url
    ));
    ws_manager
}

pub async fn run_graphql_server(ws_manager: Arc<WebSocketManager>) -> std::io::Result<()> {
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