pub mod hub;

use clap::Parser;
use hub::configuration::AppConfig;
use hub::tracing::setup_tracing;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "default")]
    env: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {

    let env = &Args::parse().env;
    let config = AppConfig::load_configuration(env)?;
    setup_tracing(&config.log_level, &config.seq_url, &config.seq_api_key)?;

    let ws_manager = hub::server::connect_websocket().await;
    hub::server::run_graphql_server(ws_manager).await?;

    Ok(())
}
