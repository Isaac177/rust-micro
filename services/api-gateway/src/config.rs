use std::{env, fmt, str::FromStr};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;



#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub app_name: String,
    pub app_env: AppEnv,
    pub log_level: String,
    pub http_bind_addr: String,
    pub nats_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_issuer: String,
}

impl GatewayConfig {
    pub fn load() -> Result<Self> {
        load_local_env();

        let app_name = required_var("APP_NAME")?;
        let app_env = required_var("APP_ENV")?
            .parse()
            .context("APP_ENV must be either 'development' or 'production'")?;
        let log_level = required_var("LOG_LEVEL")?;
        let http_bind_addr = required_var("HTTP_BIND_ADDR")?;
        let nats_url = required_var("NATS_URL")?;
        let redis_url = required_var("REDIS_URL")?;
        let jwt_secret = required_var("JWT_SECRET")?;
        let jwt_issuer = required_var("JWT_ISSUER")?;

        Ok(Self { app_name, app_env, log_level, http_bind_addr, nats_url, redis_url, jwt_secret, jwt_issuer })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum AppEnv {
    Development,
    Production,
}

impl FromStr for AppEnv {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(anyhow!("Unknown environment: {other}")),
        }
    }
}

impl fmt::Display for AppEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Development => write!(f, "development"),
            Self::Production => write!(f, "production"),
        }
    }
}

fn required_var(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("Missing environment variable {name}"))
}

fn load_local_env() {
    let candidates = [
        "services/api-gateway/.env.dev",
        ".env.dev"
    ];

    for path in candidates {
        if dotenvy::from_filename(path).is_ok() {
            break;
        }
    }
}