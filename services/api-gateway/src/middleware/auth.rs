use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{
    app::AppState,
    error::AppError,
    jwt::{self, TokenType},
};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized("missing authorization header"))?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized("invalid authorization header format"))?;

    let claims = jwt::validate_token(token, &state.config.jwt_secret, &state.config.jwt_issuer)
        .map_err(|_| AppError::Unauthorized("invalid or expired token"))?;

    if claims.token_type != TokenType::Access {
        return Err(AppError::Unauthorized("expected an access token"));
    }

    let auth_user = AuthUser {
        user_id: claims.sub,
        email: claims.email,
        display_name: claims.display_name,
    };

    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}
