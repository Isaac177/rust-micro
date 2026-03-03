
use std::env;
use anyhow::{Context, Result};
use contracts::news::list_articles::{
    ArticleSummary,
    Request as ListArticlesRequest,
    Response as ListArticlesResponse,
    SUBJECT as LIST_ARTICLES_SUBJECT,
};

use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    load_local_env();

    let nats_url = required_var("NATS_URL")?;

    let client = async_nats::connect(&nats_url)
        .await
        .with_context(|| format!("failed to connect to the NATS server"))?;

    let mut subscription = client
        .subscribe(LIST_ARTICLES_SUBJECT.to_string())
        .await
        .with_context(|| format!("failed to subscribe to {LIST_ARTICLES_SUBJECT} "))?;

    println!("news-service listening on subject {LIST_ARTICLES_SUBJECT}");

    while let Some(message) = subscription.next().await {
        let _request: ListArticlesRequest = serde_json::from_slice(&message.payload)
            .context("failed to deserialize list articles request")?;

        let response = ListArticlesResponse {
            articles: vec![
                ArticleSummary {
                    id: "article-1".to_string(),
                    title: "Rust Microservices with NATS".to_string(),
                    slug: "rust-microservices-with-nats".to_string(),
                },
                ArticleSummary {
                    id: "article-2".to_string(),
                    title: "Axum Gateway Patterns".to_string(),
                    slug: "axum-gateway-patterns".to_string(),
                },
            ],
        };

        let payload = serde_json::to_vec(&response).context("failed to serialize response")?;

        let Some(reply_subject) = message.reply else { continue; };

        client
            .publish(reply_subject, payload.into())
            .await
            .context("failed to publish list articles response")?;
    }

    Ok(())
}

fn required_var(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("missing environment variable {name}"))
}

fn load_local_env() {
    let candidates = ["services/news-service/.env.dev", ".env.dev"];

    for path in candidates {
        if dotenvy::from_filename(path).is_ok() {
            break
        }
    }
}