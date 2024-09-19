mod graphql;
mod server;
mod websocket_manager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ws_manager = server::connect_websocket().await;
    server::run_graphql_server(ws_manager).await
}
