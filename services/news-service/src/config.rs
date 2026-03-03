use std::env;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct NewsServiceConfig {
    pub nats_url: String,
    pub database_url: String,
}

impl NewsServiceConfig {
    pub fn load() -> Result<Self> {
        load_local_env();

        let nats_url = required_var("NATS_URL")?;
        let database_url = required_var("DATABASE_URL")?;

        Ok(Self {
            nats_url,
            database_url,
        })
    }
}

fn required_var(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("missing environment variable {name}"))
}

fn load_local_env() {
    let candidates = ["services/news-service/.env.dev", ".env.dev"];

    for path in candidates {
        if dotenvy::from_filename(path).is_ok() {
            break;
        }
    }
}
