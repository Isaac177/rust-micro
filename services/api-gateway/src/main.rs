mod app;
mod config;
mod error;
mod jwt;
mod nats;
mod routes;
mod telemetry;
mod middleware;

use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use app::{build_router, AppState};
use config::GatewayConfig;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Arc::new(GatewayConfig::load()?);
    telemetry::init_tracing(&config)?;

    let addr: SocketAddr = config
        .http_bind_addr
        .parse()
        .with_context(|| format!("invalid HTTP_BIND_ADDR: {}", config.http_bind_addr))?;

    let state = AppState::build(config.clone()).await?;
    let app = build_router(state);

    info!(
        app_name = %config.app_name,
        app_env = %config.app_env,
        http_bind_addr = %config.http_bind_addr,
        nats_url = %config.nats_url,
        redis_url = %config.redis_url,
        "starting api gateway"
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind TCP listener")?;

    axum::serve(listener, app)
        .await
        .context("axum server exited unexpectedly")?;

    Ok(())
}
