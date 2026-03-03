use std::time::Duration;

use contracts::news::list_articles::{
    Request as ListArticlesRequest,
    Response as ListArticlesResponse,
    SUBJECT as LIST_ARTICLES_SUBJECT,
};
use tokio::time::timeout;

use crate::{app::AppState, error::AppError};

const NEWS_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

pub async fn request_list_articles(
    state: &AppState,
    limit: i64,
    offset: i64,
) -> Result<ListArticlesResponse, AppError> {
    let payload = serde_json::to_vec(&ListArticlesRequest { limit, offset })
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
