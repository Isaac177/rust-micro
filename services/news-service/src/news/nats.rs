use std::time::Duration;

use anyhow::{Context, Result};
use async_nats::{Client, Message};
use contracts::{
    NatsResponse,
    news::{
        get_article::{
            Author, Request as GetArticleRequest, Response as GetArticleResponse,
            SUBJECT as GET_ARTICLE_SUBJECT,
        },
        list_articles::{Request as ListArticlesRequest, SUBJECT as LIST_ARTICLES_SUBJECT},
    },
    users::get_user::{Request as GetUserRequest, Response as GetUserResponse, SUBJECT as GET_USER_SUBJECT},
};
use futures::stream::select;
use futures::StreamExt;
use sqlx::PgPool;
use tokio::time::timeout;

use super::repository;

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

    let list_stream = list_sub.map(|m| ("list", m));
    let get_stream = get_sub.map(|m| ("get", m));

    let mut combined = select(list_stream, get_stream);

    while let Some((kind, message)) = combined.next().await {
        let result = match kind {
            "list" => handle_list_articles(&client, &pool, message).await,
            "get" => handle_get_article(&client, &pool, message).await,
            _ => Ok(()),
        };
        if let Err(error) = result {
            eprintln!("failed to handle {kind} request: {error:#}");
        }
    }

    Ok(())
}

async fn handle_list_articles(client: &Client, pool: &PgPool, message: Message) -> Result<()> {
    let request: ListArticlesRequest = serde_json::from_slice(message.payload.as_ref())
        .context("failed to deserialize list articles request")?;

    let response = repository::list_articles(pool, request.limit, request.offset)
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

async fn handle_get_article(client: &Client, pool: &PgPool, message: Message) -> Result<()> {
    let request: GetArticleRequest = serde_json::from_slice(message.payload.as_ref())
        .context("failed to deserialize get article request")?;

    let Some(article) = repository::get_article(pool, &request.id).await? else {
        let resp: NatsResponse<GetArticleResponse> = NatsResponse::Error {
            code: "not_found".into(),
            message: "Article not found".into(),
        };
        let payload = serde_json::to_vec(&resp).context("failed to serialize response")?;
        if let Some(reply_subject) = message.reply {
            client.publish(reply_subject, payload.into()).await?;
        }
        return Ok(());
    };

    // Call user-service directly over NATS to get author details
    let author = fetch_author(client, &article.author_user_id).await?;

    let resp: NatsResponse<GetArticleResponse> = NatsResponse::Ok {
        data: GetArticleResponse {
            id: article.id,
            slug: article.slug,
            title: article.title,
            summary: article.summary,
            body_markdown: article.body_markdown,
            body_html: article.body_html,
            cover_image_url: article.cover_image_url,
            status: article.status,
            published_at: article.published_at,
            created_at: article.created_at,
            updated_at: article.updated_at,
            author,
        },
    };

    let payload = serde_json::to_vec(&resp).context("failed to serialize get article response")?;

    if let Some(reply_subject) = message.reply {
        client
            .publish(reply_subject, payload.into())
            .await
            .context("failed to publish get article response")?;
    }

    Ok(())
}

async fn fetch_author(client: &Client, user_id: &str) -> Result<Author> {
    let payload = serde_json::to_vec(&GetUserRequest {
        user_id: user_id.to_string(),
    })
    .context("failed to serialize get user request")?;

    let message = timeout(
        Duration::from_secs(2),
        client.request(GET_USER_SUBJECT.to_string(), payload.into()),
    )
    .await
    .context("user-service request timed out")?
    .context("user-service request failed")?;

    let response: NatsResponse<GetUserResponse> =
        serde_json::from_slice(&message.payload).context("failed to deserialize user response")?;

    match response {
        NatsResponse::Ok { data } => Ok(Author {
            id: data.id,
            email: data.email,
            display_name: data.display_name,
        }),
        NatsResponse::Error { code, message } => {
            anyhow::bail!("user-service error {code}: {message}")
        }
    }
}
