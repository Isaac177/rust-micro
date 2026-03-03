use axum::extract::State;
use crate::app::AppState;
use crate::error::AppError;

pub async fn health_check(State(state): State<AppState>) -> Result<&'static str, AppError> {
    let _app_name = &state.config.app_name;

    Ok("Ok")
}

pub async fn readiness_check(State(state): State<AppState>) -> Result<&'static str, AppError> {
    state.nats_client.flush().await.map_err(|_| AppError::UpstreamUnavailable("nats is unavailable"))?;

    let mut redis_connection = state.redis_client.get_multiplexed_async_connection().await.map_err(|_| AppError::UpstreamUnavailable("redis is unavailable"))?;

    let pong: String = redis::cmd("PING").query_async(&mut redis_connection).await.map_err(|_| AppError::UpstreamUnavailable("redis ping failed"))?;

    if pong != "PONG" {
        return Err(AppError::UpstreamUnavailable("redis returned an invalid ping response"))
    }

    Ok("READY")

}