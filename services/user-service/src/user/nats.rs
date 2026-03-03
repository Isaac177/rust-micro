use anyhow::{Context, Result};
use async_nats::{Client, Message};
use contracts::users::list_users::{
    Request as ListUsersRequest,
    SUBJECT as LIST_USERS_SUBJECT,
};
use futures::StreamExt;
use sqlx::PgPool;

use super::repository;

pub async fn serve(client: Client, pool: PgPool) -> Result<()> {
    let mut subscription = client
        .subscribe(LIST_USERS_SUBJECT.to_string())
        .await
        .with_context(|| format!("failed to subscribe to {LIST_USERS_SUBJECT}"))?;

    println!("user-service listening on subject {LIST_USERS_SUBJECT}");

    while let Some(message) = subscription.next().await {
        if let Err(error) = handle_list_users(&client, &pool, message).await {
            eprintln!("failed to handle {LIST_USERS_SUBJECT}: {error:#}");
        }
    }

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
