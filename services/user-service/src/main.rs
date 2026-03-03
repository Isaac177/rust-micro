mod config;
mod user;

use anyhow::{Context, Result};
use config::UserServiceConfig;
use sqlx::{migrate::Migrator, PgPool};

static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<()> {
    let config = UserServiceConfig::load()?;

    let pool = PgPool::connect(&config.database_url)
        .await
        .with_context(|| "failed to connect to PostgreSQL")?;

    MIGRATOR
        .run(&pool)
        .await
        .with_context(|| "failed to run PostgreSQL migrations")?;

    let nats_client = async_nats::connect(&config.nats_url)
        .await
        .with_context(|| format!("failed to connect to NATS at {}", config.nats_url))?;

    user::serve(nats_client, pool).await
}
