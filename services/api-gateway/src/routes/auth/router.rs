use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    app::AppState,
    error::AppError,
    jwt::{self, TokenPair, TokenType},
};

use super::nats;

#[derive(Debug, Deserialize)]
struct RegisterBody {
    email: String,
    password: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct LoginBody {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct RefreshBody {
    refresh_token: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh))
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<TokenPair>, AppError> {
    if body.email.is_empty() || body.password.is_empty() || body.display_name.is_empty() {
        return Err(AppError::BadRequest("email, password, and display_name are required"));
    }

    if body.password.len() < 8 {
        return Err(AppError::BadRequest("password must be at least 8 characters"));
    }

    let user = nats::request_register(&state, body.email, body.password, body.display_name).await?;

    let tokens = jwt::mint_token_pair(
        &user.id,
        &user.email,
        &user.display_name,
        &state.config.jwt_secret,
        &state.config.jwt_issuer,
    )
    .map_err(|_| AppError::Internal("failed to mint tokens"))?;

    Ok(Json(tokens))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Json<TokenPair>, AppError> {
    if body.email.is_empty() || body.password.is_empty() {
        return Err(AppError::BadRequest("email and password are required"));
    }

    let user = nats::request_authenticate(&state, body.email, body.password).await?;

    let tokens = jwt::mint_token_pair(
        &user.id,
        &user.email,
        &user.display_name,
        &state.config.jwt_secret,
        &state.config.jwt_issuer,
    )
    .map_err(|_| AppError::Internal("failed to mint tokens"))?;

    Ok(Json(tokens))
}

async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshBody>,
) -> Result<Json<TokenPair>, AppError> {
    let claims = jwt::validate_token(
        &body.refresh_token,
        &state.config.jwt_secret,
        &state.config.jwt_issuer,
    )
    .map_err(|_| AppError::Unauthorized("invalid or expired refresh token"))?;

    if claims.token_type != TokenType::Refresh {
        return Err(AppError::Unauthorized("expected a refresh token"));
    }

    let tokens = jwt::mint_token_pair(
        &claims.sub,
        &claims.email,
        &claims.display_name,
        &state.config.jwt_secret,
        &state.config.jwt_issuer,
    )
    .map_err(|_| AppError::Internal("failed to mint tokens"))?;

    Ok(Json(tokens))
}
