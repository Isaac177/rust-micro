use std::time::Duration;

use contracts::{
    NatsResponse,
    users::{
        authenticate::{
            Request as AuthenticateRequest, Response as AuthenticateResponse,
            SUBJECT as AUTHENTICATE_SUBJECT,
        },
        register::{
            Request as RegisterRequest, Response as RegisterResponse,
            SUBJECT as REGISTER_SUBJECT,
        },
    },
};

use crate::{app::AppState, error::AppError, nats};

const AUTH_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn request_register(
    state: &AppState,
    email: String,
    password: String,
    display_name: String,
) -> Result<RegisterResponse, AppError> {
    let response: NatsResponse<RegisterResponse> = nats::request_json(
        state,
        REGISTER_SUBJECT,
        &RegisterRequest {
            email,
            password,
            display_name,
        },
        AUTH_REQUEST_TIMEOUT,
        "user-service register request timed out",
        "user-service register request failed",
        "can't serialize register request",
        "can't deserialize register response",
    )
    .await?;

    match response {
        NatsResponse::Ok { data } => Ok(data),
        NatsResponse::Error { code, .. } => match code.as_str() {
            "email_taken" => Err(AppError::BadRequest("A user with this email already exists")),
            _ => Err(AppError::Internal("unexpected error from user-service")),
        },
    }
}

pub async fn request_authenticate(
    state: &AppState,
    email: String,
    password: String,
) -> Result<AuthenticateResponse, AppError> {
    let response: NatsResponse<AuthenticateResponse> = nats::request_json(
        state,
        AUTHENTICATE_SUBJECT,
        &AuthenticateRequest { email, password },
        AUTH_REQUEST_TIMEOUT,
        "user-service authenticate request timed out",
        "user-service authenticate request failed",
        "can't serialize authenticate request",
        "can't deserialize authenticate response",
    )
    .await?;

    match response {
        NatsResponse::Ok { data } => Ok(data),
        NatsResponse::Error { code, .. } => match code.as_str() {
            "invalid_credentials" => {
                Err(AppError::Unauthorized("Invalid email or password"))
            }
            "account_disabled" => {
                Err(AppError::Forbidden("This account has been disabled"))
            }
            _ => Err(AppError::Internal("unexpected error from user-service")),
        },
    }
}
