use anyhow::{Context, Result};
use async_nats::{Client, Message};
use contracts::news::{
    get_article::SUBJECT as GET_ARTICLE_SUBJECT,
    list_articles::SUBJECT as LIST_ARTICLES_SUBJECT,
};
use futures::stream::select;
use futures::StreamExt;
use sqlx::PgPool;

use super::handlers;

pub async fn serve(client: Client, pool: PgPool) -> Result<()> {
    let list_sub = client
        .subscribe(LIST_ARTICLES_SUBJECT.to_string())
        .await
        .with_context(|| format!("failed to subscribe to {LIST_ARTICLES_SUBJECT}"))?;

    let get_sub = client
        .subscribe(GET_ARTICLE_SUBJECT.to_string())
        .await
        .with_context(|| format!("failed to subscribe to {GET_ARTICLE_SUBJECT}"))?;

    println!("news-service listening on subjects: {LIST_ARTICLES_SUBJECT}, {GET_ARTICLE_SUBJECT}");

    let mut combined = select(list_sub.map(|m| ("list", m)), get_sub.map(|m| ("get", m)));

    while let Some((kind, message)) = combined.next().await {
        let result = match kind {
            "list" => reply(&client, &message, handlers::list_articles(&pool, &message.payload).await).await,
            "get"  => reply(&client, &message, handlers::get_article(&client, &pool, &message.payload).await).await,
            _      => Ok(()),
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
