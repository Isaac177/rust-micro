use std::time::Duration;

use tokio::time::timeout;

use crate::{app::AppState, error::AppError};

use super::dto::{ListArticlesRequest, ListArticlesResponse};

const LIST_ARTICLES_SUBJECT: &str = "news.articles.list";
const NEWS_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

pub async fn request_list_articles(state: &AppState) -> Result<ListArticlesResponse, AppError> {
    let payload = serde_json::to_vec(&ListArticlesRequest {})
        .map_err(|_| AppError::Internal("can't serialize list articles request"))?;

    let message = timeout(
        NEWS_REQUEST_TIMEOUT,
        state
            .nats_client
            .request(LIST_ARTICLES_SUBJECT.to_string(), payload.into()),
    )
        .await
        .map_err(|_| AppError::UpstreamUnavailable("news-service request timed out"))?
        .map_err(|_| AppError::UpstreamUnavailable("news-service request failed"))?;

    serde_json::from_slice::<ListArticlesResponse>(&message.payload)
        .map_err(|_| AppError::Internal("can't deserialize list articles response"))
}
