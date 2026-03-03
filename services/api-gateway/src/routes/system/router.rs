use axum::{extract::State, routing::get, Json, Router};

use crate::{
    app::AppState,
    error::{AppError, StatusResponse},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
}

async fn health_check(State(state): State<AppState>) -> Result<Json<StatusResponse>, AppError> {
    let _ = &state.config.app_name;

    Ok(Json(StatusResponse { status: "ok" }))
}

async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>, AppError> {
    state
        .nats_client
        .flush()
        .await
        .map_err(|_| AppError::UpstreamUnavailable("nats is unavailable"))?;

    let mut redis_connection = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| AppError::UpstreamUnavailable("redis is unavailable"))?;

    let pong: String = redis::cmd("PING")
        .query_async(&mut redis_connection)
        .await
        .map_err(|_| AppError::UpstreamUnavailable("redis ping failed"))?;

    if pong != "PONG" {
        return Err(AppError::UpstreamUnavailable(
            "redis returned an invalid ping response",
        ));
    }

    Ok(Json(StatusResponse { status: "ready" }))
}
