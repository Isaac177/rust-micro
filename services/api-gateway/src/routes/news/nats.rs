use std::time::Duration;

use contracts::news::list_articles::{
    Request as ListArticlesRequest,
    Response as ListArticlesResponse,
    SUBJECT as LIST_ARTICLES_SUBJECT,
};

use crate::{app::AppState, error::AppError, nats};

const NEWS_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

pub async fn request_list_articles(
    state: &AppState,
    limit: i64,
    offset: i64,
) -> Result<ListArticlesResponse, AppError> {
    nats::request_json(
        state,
        LIST_ARTICLES_SUBJECT,
        &ListArticlesRequest { limit, offset },
        NEWS_REQUEST_TIMEOUT,
        "news-service request timed out",
        "news-service request failed",
        "can't serialize list articles request",
        "can't deserialize list articles response",
    )
    .await
}
