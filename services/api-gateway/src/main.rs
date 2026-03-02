mod config;
mod telemetry;
mod error;

use anyhow::Result;
use tracing::info;
use config::GatewayConfig;
use error::AppError;

fn main() -> Result<()> {
    let config = GatewayConfig::load()?;
    telemetry::init_tracing(&config)?;

    info!(
        app_name = %config.app_name,
        app_env = %config.app_env,
        http_bind_addr = %config.http_bind_addr,
        nats_url = %config.nats_url,
        redis_url = %config.redis_url,
        "api gateway configuration loaded"
    );

    let _example_error = AppError::Internal("example");

    Ok(())
}
