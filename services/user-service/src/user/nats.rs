use anyhow::{Context, Result};
use async_nats::{Client, Message};
use contracts::users::{
    authenticate::SUBJECT as AUTHENTICATE_SUBJECT,
    get_user::SUBJECT as GET_USER_SUBJECT,
    list_users::SUBJECT as LIST_USERS_SUBJECT,
    register::SUBJECT as REGISTER_SUBJECT,
};
use futures::stream::select;
use futures::StreamExt;
use sqlx::PgPool;

use super::handlers;

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

    let mut combined = select(
        select(list_sub.map(|m| ("list", m)), register_sub.map(|m| ("register", m))),
        select(auth_sub.map(|m| ("auth", m)), get_user_sub.map(|m| ("get_user", m))),
    );

    while let Some((kind, message)) = combined.next().await {
        let result = match kind {
            "list"     => reply(&client, &message, handlers::list_users(&pool, &message.payload).await).await,
            "register" => reply(&client, &message, handlers::register(&pool, &message.payload).await).await,
            "auth"     => reply(&client, &message, handlers::authenticate(&pool, &message.payload).await).await,
            "get_user" => reply(&client, &message, handlers::get_user(&pool, &message.payload).await).await,
            _          => Ok(()),
        };
        if let Err(error) = result {
            eprintln!("failed to handle {kind} request: {error:#}");
        }
    }

    Ok(())
}

async fn reply(client: &Client, message: &Message, result: Result<Vec<u8>>) -> Result<()> {
    let payload = result?;
    let Some(reply_subject) = &message.reply else {
        return Ok(());
    };
    client
        .publish(reply_subject.clone(), payload.into())
        .await
        .context("failed to publish response")
}
