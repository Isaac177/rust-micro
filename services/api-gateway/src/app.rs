use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{middleware, Router};

use crate::{config::GatewayConfig, routes};
use crate::middleware::{cors, request_id};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<GatewayConfig>,
    pub nats_client: async_nats::Client,
    pub redis_client: redis::Client,
}

impl AppState {
    pub async fn build(config: Arc<GatewayConfig>) -> Result<Self> {
        let nats_client = async_nats::connect(&config.nats_url)
            .await
            .with_context(|| format!("failed to connect to NATS at {}", config.nats_url))?;

        let redis_client = redis::Client::open(config.redis_url.clone())
            .with_context(|| format!("failed to create Redis client for {}", config.redis_url))?;

        Ok(Self {
            config,
            nats_client,
            redis_client,
        })
    }
}

pub fn build_router(state: AppState) -> Router {
    routes::router()
        .layer(cors::build_cors())
        .layer(middleware::from_fn(request_id::set_request_id))
        .with_state(state)
}
