use anyhow::{Context, Result};
use contracts::{
    NatsResponse,
    users::{
        authenticate::{Request as AuthenticateRequest, Response as AuthenticateResponse},
        get_user::{Request as GetUserRequest, Response as GetUserResponse},
        list_users::{Request as ListUsersRequest, Response as ListUsersResponse},
        register::{Request as RegisterRequest, Response as RegisterResponse},
    },
};
use sqlx::PgPool;

use super::{password, repository};

pub async fn list_users(pool: &PgPool, payload: &[u8]) -> Result<Vec<u8>> {
    let req: ListUsersRequest =
        serde_json::from_slice(payload).context("failed to deserialize list users request")?;

    let response: ListUsersResponse =
        repository::list_users(pool, req.limit, req.offset).await?;

    serde_json::to_vec(&response).context("failed to serialize list users response")
}

pub async fn register(pool: &PgPool, payload: &[u8]) -> Result<Vec<u8>> {
    let req: RegisterRequest =
        serde_json::from_slice(payload).context("failed to deserialize register request")?;

    let resp: NatsResponse<RegisterResponse> =
        if repository::find_user_by_email(pool, &req.email).await?.is_some() {
            NatsResponse::Error {
                code: "email_taken".into(),
                message: "A user with this email already exists".into(),
            }
        } else {
            let password_hash = password::hash_password(&req.password)?;
            let user_id = uuid::Uuid::now_v7();
            repository::create_user(pool, user_id, &req.email, &req.display_name, &password_hash).await?;
            NatsResponse::Ok {
                data: RegisterResponse {
                    id: user_id.to_string(),
                    email: req.email,
                    display_name: req.display_name,
                },
            }
        };

    serde_json::to_vec(&resp).context("failed to serialize register response")
}

pub async fn authenticate(pool: &PgPool, payload: &[u8]) -> Result<Vec<u8>> {
    let req: AuthenticateRequest =
        serde_json::from_slice(payload).context("failed to deserialize authenticate request")?;

    let resp: NatsResponse<AuthenticateResponse> =
        match repository::find_user_by_email(pool, &req.email).await? {
            None => NatsResponse::Error {
                code: "invalid_credentials".into(),
                message: "Invalid email or password".into(),
            },
            Some(user) if user.status != "active" => NatsResponse::Error {
                code: "account_disabled".into(),
                message: "This account has been disabled".into(),
            },
            Some(user) if !password::verify_password(&req.password, &user.password_hash)? => {
                NatsResponse::Error {
                    code: "invalid_credentials".into(),
                    message: "Invalid email or password".into(),
                }
            }
            Some(user) => NatsResponse::Ok {
                data: AuthenticateResponse {
                    id: user.id,
                    email: user.email,
                    display_name: user.display_name,
                },
            },
        };

    serde_json::to_vec(&resp).context("failed to serialize authenticate response")
}

pub async fn get_user(pool: &PgPool, payload: &[u8]) -> Result<Vec<u8>> {
    let req: GetUserRequest =
        serde_json::from_slice(payload).context("failed to deserialize get user request")?;

    let resp: NatsResponse<GetUserResponse> = match repository::find_by_id(pool, &req.user_id).await? {
        Some(u) => NatsResponse::Ok {
            data: GetUserResponse {
                id: u.id,
                email: u.email,
                display_name: u.display_name,
            },
        },
        None => NatsResponse::Error {
            code: "not_found".into(),
            message: "User not found".into(),
        },
    };

    serde_json::to_vec(&resp).context("failed to serialize get user response")
}
