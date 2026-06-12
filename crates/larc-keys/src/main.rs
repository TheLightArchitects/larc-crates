use anyhow::Result;
use axum::{
    Router,
    middleware,
    routing::{delete, get, post},
};
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub config: config::Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_env("RUST_LOG"))
        .with_writer(std::io::stderr)
        .init();

    let config = config::Config::from_env()?;
    let conn = db::open(&config.database_path)?;
    let addr = format!("{}:{}", config.host, config.port);

    let state = Arc::new(AppState {
        db: Mutex::new(conn),
        config,
    });

    let authed = Router::new()
        .route("/v1/auth/me", get(handlers::users::me))
        .route("/v1/keys", get(handlers::keys::list))
        .route("/v1/keys", post(handlers::keys::create))
        .route("/v1/keys/{id}", delete(handlers::keys::revoke))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            handlers::users::require_auth,
        ));

    let public = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/v1/auth/register", post(handlers::users::register))
        .route("/v1/auth/login", post(handlers::users::login))
        .route("/v1/mcp/tools", get(handlers::mcp::list_tools))
        .route("/v1/mcp/call", post(handlers::mcp::call_tool));

    let app = Router::new()
        .merge(public)
        .merge(authed)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    info!("larc-keys listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
