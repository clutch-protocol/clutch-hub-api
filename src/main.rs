pub mod hub;

use clap::Parser;
use hub::configuration::AppConfig;
use hub::metric::serve_metrics;
use hub::tracing::setup_tracing;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "default")]
    env: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = &Args::parse().env;
    let config = AppConfig::load_configuration(env)?;

    setup_tracing(&config.log_level, &config.seq_url, &config.seq_api_key)?;
    serve_metrics(&config.serve_metric_addr);
    let ws_manager = hub::server::connect_websocket(&config.clutch_node_wss_url).await;
    hub::server::run_graphql_server(ws_manager).await?;

    Ok(())
}
