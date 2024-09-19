
use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use crate::hub::graphql::handler::graphql_handler;
use crate::hub::graphql::build_schema;
use crate::hub::websocket_manager::WebSocketManager;

pub async fn connect_websocket() -> Arc<WebSocketManager> {
    let ws_manager = Arc::new(WebSocketManager::new("wss://your-blockchain-node-url".to_string()));
    ws_manager.connect().await;
    ws_manager
}

pub async fn run_graphql_server(ws_manager: Arc<WebSocketManager>) -> std::io::Result<()> {
    let schema = build_schema();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .app_data(web::Data::new(ws_manager.clone())) // Share the WebSocket manager
            .service(web::resource("/graphql").route(web::post().to(graphql_handler)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

