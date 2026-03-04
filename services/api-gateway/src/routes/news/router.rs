use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use contracts::news::{
    get_article::Response as GetArticleResponse,
    list_articles::Response as ListArticlesResponse,
};
use serde::Deserialize;

use crate::{app::AppState, error::AppError};

use super::nats;

const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 100;

#[derive(Debug, Deserialize)]
struct ListArticlesQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/news/articles", get(list_articles))
        .route("/api/v1/news/articles/{id}", get(get_article))
}

async fn list_articles(
    State(state): State<AppState>,
    Query(query): Query<ListArticlesQuery>,
) -> Result<Json<ListArticlesResponse>, AppError> {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = query.offset.unwrap_or(0);

    if offset < 0 {
        return Err(AppError::BadRequest(
            "offset must be greater than or equal to 0",
        ));
    }

    let response = nats::request_list_articles(&state, limit, offset).await?;

    Ok(Json(response))
}

async fn get_article(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GetArticleResponse>, AppError> {
    let response = nats::request_get_article(&state, id).await?;
    Ok(Json(response))
}
