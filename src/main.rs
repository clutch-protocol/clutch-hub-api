pub mod hub;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ws_manager = hub::server::connect_websocket().await;
    hub::server::run_graphql_server(ws_manager).await
}
