mod config;
mod observability;

use anyhow::Result;
use tracing::info;
use config::GatewayConfig;

fn main() -> Result<()> {
    let config = GatewayConfig::load()?;
    observability::telemetry::init_tracing(&config)?;

    info!(
        app_name = %config.app_name,
        app_env = %config.app_env,
        http_bind_addr = %config.http_bind_addr,
        nats_url = %config.nats_url,
        redis_url = %config.redis_url,
        "api gateway configuration loaded"
    );
    Ok(())
}
