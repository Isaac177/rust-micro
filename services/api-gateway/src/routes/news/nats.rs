use std::time::Duration;

use contracts::news::{
    get_article::{
        Request as GetArticleRequest, Response as GetArticleResponse,
        SUBJECT as GET_ARTICLE_SUBJECT,
    },
    list_articles::{
        Request as ListArticlesRequest, Response as ListArticlesResponse,
        SUBJECT as LIST_ARTICLES_SUBJECT,
    },
};
use contracts::NatsResponse;

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

pub async fn request_get_article(
    state: &AppState,
    id: String,
) -> Result<GetArticleResponse, AppError> {
    let response: NatsResponse<GetArticleResponse> = nats::request_json(
        state,
        GET_ARTICLE_SUBJECT,
        &GetArticleRequest { id },
        NEWS_REQUEST_TIMEOUT,
        "news-service request timed out",
        "news-service request failed",
        "can't serialize get article request",
        "can't deserialize get article response",
    )
    .await?;

    match response {
        NatsResponse::Ok { data } => Ok(data),
        NatsResponse::Error { code, .. } => match code.as_str() {
            "not_found" => Err(AppError::NotFound("Article not found")),
            _ => Err(AppError::Internal("unexpected error from news-service")),
        },
    }
}
