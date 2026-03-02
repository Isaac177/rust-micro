use anyhow::{Context, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::GatewayConfig;

pub fn init_tracing(config: &GatewayConfig) -> Result<()> {
    let env_filter = EnvFilter::try_new(&config.log_level)
        .with_context(|| format!("Failed to parse log level \"{}\"", config.log_level))?;

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .context("Failed to initialise tracing logger")?;

    Ok(())
}
