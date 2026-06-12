use anyhow::Result;
use rmcp::{ServiceExt, transport::io::stdio};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod proxy;

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_env("RUST_LOG"))
        .with_writer(std::io::stderr)
        .init();

    let config = config::Config::from_env()?;

    info!(
        version = env!("CARGO_PKG_VERSION"),
        api_url = %config.api_url,
        "larc-proxy starting"
    );

    let handler = proxy::ProxyHandler::new(config)?;
    let server = handler.serve(stdio()).await?;

    info!("larc-proxy connected — waiting for requests");
    server.waiting().await?;

    Ok(())
}
