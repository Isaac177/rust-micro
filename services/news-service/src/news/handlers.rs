use std::time::Duration;

use anyhow::{Context, Result};
use async_nats::Client;
use contracts::{
    NatsResponse,
    news::{
        get_article::{Author, Request as GetArticleRequest, Response as GetArticleResponse},
        list_articles::{Request as ListArticlesRequest, Response as ListArticlesResponse},
    },
    users::get_user::{Request as GetUserRequest, Response as GetUserResponse, SUBJECT as GET_USER_SUBJECT},
};
use sqlx::PgPool;
use tokio::time::timeout;

use super::repository;

pub async fn list_articles(pool: &PgPool, payload: &[u8]) -> Result<Vec<u8>> {
    let req: ListArticlesRequest =
        serde_json::from_slice(payload).context("failed to deserialize list articles request")?;

    let response: ListArticlesResponse =
        repository::list_articles(pool, req.limit, req.offset).await?;

    serde_json::to_vec(&response).context("failed to serialize list articles response")
}

pub async fn get_article(client: &Client, pool: &PgPool, payload: &[u8]) -> Result<Vec<u8>> {
    let req: GetArticleRequest =
        serde_json::from_slice(payload).context("failed to deserialize get article request")?;

    let resp: NatsResponse<GetArticleResponse> = match repository::get_article(pool, &req.id).await? {
        None => NatsResponse::Error {
            code: "not_found".into(),
            message: "Article not found".into(),
        },
        Some(article) => {
            let author = fetch_author(client, &article.author_user_id).await?;
            NatsResponse::Ok {
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
            }
        }
    };

    serde_json::to_vec(&resp).context("failed to serialize get article response")
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
