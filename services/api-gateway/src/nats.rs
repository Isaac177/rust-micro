use std::time::Duration;

use serde::{de::DeserializeOwned, Serialize};
use tokio::time::timeout;

use crate::{app::AppState, error::AppError};

pub async fn request_json<TRequest, TResponse>(
    state: &AppState,
    subject: &str,
    payload: &TRequest,
    timeout_duration: Duration,
    request_timeout_message: &'static str,
    request_failed_message: &'static str,
    serialize_error_message: &'static str,
    deserialize_error_message: &'static str,
) -> Result<TResponse, AppError>
where
    TRequest: Serialize,
    TResponse: DeserializeOwned,
{
    let payload = serde_json::to_vec(payload)
        .map_err(|_| AppError::Internal(serialize_error_message))?;

    let message = timeout(
        timeout_duration,
        state.nats_client.request(subject.to_string(), payload.into()),
    )
    .await
    .map_err(|_| AppError::UpstreamUnavailable(request_timeout_message))?
    .map_err(|_| AppError::UpstreamUnavailable(request_failed_message))?;

    serde_json::from_slice::<TResponse>(&message.payload)
        .map_err(|_| AppError::Internal(deserialize_error_message))
}
