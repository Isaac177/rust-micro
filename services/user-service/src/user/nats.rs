use anyhow::{Context, Result};
use async_nats::{Client, Message};
use contracts::{
    NatsResponse,
    users::{
        authenticate::{
            Request as AuthenticateRequest, Response as AuthenticateResponse,
            SUBJECT as AUTHENTICATE_SUBJECT,
        },
        get_user::{
            Request as GetUserRequest, Response as GetUserResponse,
            SUBJECT as GET_USER_SUBJECT,
        },
        list_users::{Request as ListUsersRequest, SUBJECT as LIST_USERS_SUBJECT},
        register::{
            Request as RegisterRequest, Response as RegisterResponse,
            SUBJECT as REGISTER_SUBJECT,
        },
    },
};
use futures::stream::select;
use futures::StreamExt;
use sqlx::PgPool;

use super::{password, repository};

pub async fn serve(client: Client, pool: PgPool) -> Result<()> {
    let list_sub = client
        .subscribe(LIST_USERS_SUBJECT.to_string())
        .await
        .with_context(|| format!("failed to subscribe to {LIST_USERS_SUBJECT}"))?;

    let register_sub = client
        .subscribe(REGISTER_SUBJECT.to_string())
        .await
        .with_context(|| format!("failed to subscribe to {REGISTER_SUBJECT}"))?;

    let auth_sub = client
        .subscribe(AUTHENTICATE_SUBJECT.to_string())
        .await
        .with_context(|| format!("failed to subscribe to {AUTHENTICATE_SUBJECT}"))?;

    let get_user_sub = client
        .subscribe(GET_USER_SUBJECT.to_string())
        .await
        .with_context(|| format!("failed to subscribe to {GET_USER_SUBJECT}"))?;

    println!("user-service listening on subjects: {LIST_USERS_SUBJECT}, {REGISTER_SUBJECT}, {AUTHENTICATE_SUBJECT}, {GET_USER_SUBJECT}");

    let list_stream = list_sub.map(|m| ("list", m));
    let register_stream = register_sub.map(|m| ("register", m));
    let auth_stream = auth_sub.map(|m| ("auth", m));
    let get_user_stream = get_user_sub.map(|m| ("get_user", m));

    let mut combined = select(select(list_stream, register_stream), select(auth_stream, get_user_stream));

    while let Some((kind, message)) = combined.next().await {
        let result = match kind {
            "list" => handle_list_users(&client, &pool, message).await,
            "register" => handle_register(&client, &pool, message).await,
            "auth" => handle_authenticate(&client, &pool, message).await,
            "get_user" => handle_get_user(&client, &pool, message).await,
            _ => Ok(()),
        };
        if let Err(error) = result {
            eprintln!("failed to handle {kind} request: {error:#}");
        }
    }
    Ok(())
}

async fn reply<T: serde::Serialize>(
    client: &Client,
    message: &Message,
    response: &NatsResponse<T>,
) -> Result<()> {
    let payload = serde_json::to_vec(response).context("failed to serialize response")?;

    let Some(reply_subject) = &message.reply else {
        return Ok(());
    };

    client
        .publish(reply_subject.clone(), payload.into())
        .await
        .context("failed to publish response")?;
    Ok(())
}

async fn handle_list_users(client: &Client, pool: &PgPool, message: Message) -> Result<()> {
    let request: ListUsersRequest = serde_json::from_slice(message.payload.as_ref())
        .context("failed to deserialize list users request")?;

    let response = repository::list_users(pool, request.limit, request.offset)
        .await
        .context("failed to load users from database")?;

    let payload = serde_json::to_vec(&response)
        .context("failed to serialize list users response")?;

    let Some(reply_subject) = message.reply else {
        return Ok(());
    };

    client
        .publish(reply_subject, payload.into())
        .await
        .context("failed to publish list users response")?;

    Ok(())
}

async fn handle_register(client: &Client, pool: &PgPool, message: Message) -> Result<()> {
    let request: RegisterRequest = serde_json::from_slice(message.payload.as_ref())
        .context("failed to deserialize register request")?;

    if repository::find_user_by_email(pool, &request.email).await?.is_some() {
        let resp: NatsResponse<RegisterResponse> = NatsResponse::Error {
            code: "email_taken".into(),
            message: "A user with this email already exists".into(),
        };
        return reply(client, &message, &resp).await;
    }

    let password_hash = password::hash_password(&request.password)?;
    let user_id = uuid::Uuid::now_v7();

    repository::create_user(pool, user_id, &request.email, &request.display_name, &password_hash)
        .await
        .context("failed to create user in database")?;

    let resp: NatsResponse<RegisterResponse> = NatsResponse::Ok {
        data: RegisterResponse {
            id: user_id.to_string(),
            email: request.email,
            display_name: request.display_name,
        },
    };

    reply(client, &message, &resp).await
}

async fn handle_authenticate(client: &Client, pool: &PgPool, message: Message) -> Result<()> {
    let request: AuthenticateRequest = serde_json::from_slice(message.payload.as_ref())
        .context("failed to deserialize authenticate request")?;

    let user = repository::find_user_by_email(pool, &request.email).await?;

    let Some(user) = user else {
        let resp: NatsResponse<AuthenticateResponse> = NatsResponse::Error {
            code: "invalid_credentials".into(),
            message: "Invalid email or password".into(),
        };
        return reply(client, &message, &resp).await;
    };

    if user.status != "active" {
        let resp: NatsResponse<AuthenticateResponse> = NatsResponse::Error {
            code: "account_disabled".into(),
            message: "This account has been disabled".into(),
        };
        return reply(client, &message, &resp).await;
    }

    let valid = password::verify_password(&request.password, &user.password_hash)?;

    if !valid {
        let resp: NatsResponse<AuthenticateResponse> = NatsResponse::Error {
            code: "invalid_credentials".into(),
            message: "Invalid email or password".into(),
        };
        return reply(client, &message, &resp).await;
    }

    let resp: NatsResponse<AuthenticateResponse> = NatsResponse::Ok {
        data: AuthenticateResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        },
    };

    reply(client, &message, &resp).await
}

async fn handle_get_user(client: &Client, pool: &PgPool, message: Message) -> Result<()> {
    let request: GetUserRequest = serde_json::from_slice(message.payload.as_ref())
        .context("failed to deserialize get user request")?;

    let user = repository::find_by_id(pool, &request.user_id).await?;

    let resp: NatsResponse<GetUserResponse> = match user {
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

    reply(client, &message, &resp).await
}
