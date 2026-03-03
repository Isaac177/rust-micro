use anyhow::{Context, Result};
use async_nats::{Client, Message};
use contracts::news::list_articles::{
    Request as ListArticlesRequest,
    SUBJECT as LIST_ARTICLES_SUBJECT,
};
use futures::StreamExt;
use sqlx::PgPool;

use super::repository;

pub async fn serve(client: Client, pool: PgPool) -> Result<()> {
    let mut subscription = client
        .subscribe(LIST_ARTICLES_SUBJECT.to_string())
        .await
        .with_context(|| format!("failed to subscribe to {LIST_ARTICLES_SUBJECT}"))?;

    println!("news-service listening on subject {LIST_ARTICLES_SUBJECT}");

    while let Some(message) = subscription.next().await {
        if let Err(error) = handle_list_articles(&client, &pool, message).await {
            eprintln!("failed to handle {LIST_ARTICLES_SUBJECT}: {error:#}");
        }
    }

    Ok(())
}

async fn handle_list_articles(client: &Client, pool: &PgPool, message: Message) -> Result<()> {
    let _request: ListArticlesRequest = serde_json::from_slice(message.payload.as_ref())
        .context("failed to deserialize list articles request")?;

    let response = repository::list_articles(pool)
        .await
        .context("failed to load articles from database")?;

    let payload = serde_json::to_vec(&response)
        .context("failed to serialize list articles response")?;

    let Some(reply_subject) = message.reply else {
        return Ok(());
    };

    client
        .publish(reply_subject, payload.into())
        .await
        .context("failed to publish list articles response")?;

    Ok(())
}
